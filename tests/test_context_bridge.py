"""Document lifetime parsing, in-operation binding, and native guard routing."""

import sys
from pathlib import Path

import pytest

from inkscape_mcp.config import Settings
from inkscape_mcp.live import context_bridge, managed_dbus
from inkscape_mcp.live.context_bridge import ENV_BRIDGE, list_contexts, read_context
from inkscape_mcp.live.managed_dbus import ENV_STDOUT, ManagedDBusTransport
from inkscape_mcp.live.transport import LiveDocumentRef, LiveError, LiveSelection

WINDOW_A = "11111111-1111-1111-1111-111111111111"
WINDOW_B = "22222222-2222-2222-2222-222222222222"
DOC_A = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"
DOC_B = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"


def test_identical_drawings_keep_distinct_runtime_identities(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    rows = [(WINDOW_A, DOC_A, "same.svg"), (WINDOW_B, DOC_B, "same.svg")]
    monkeypatch.setattr(context_bridge, "context_call", lambda *args: repr((rows,)))
    docs = list_contexts(1)
    assert docs[0].name == docs[1].name
    assert docs[0].document_id != docs[1].document_id
    assert docs[0].window_id != docs[1].window_id


@pytest.mark.parametrize(
    "reply", ["('', '', '')", "('file.svg', 'hash', 'title')", "{}", "(1,2,3)"]
)
def test_unknown_identity_is_never_a_task_binding(
    reply: str, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setattr(context_bridge, "context_call", lambda *args: reply)
    with pytest.raises(LiveError):
        read_context(1)


def test_empty_native_document_list(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(context_bridge, "context_call", lambda *args: "(@a(sss) [],)")
    assert list_contexts(1) == []


@pytest.fixture
def bridge(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> ManagedDBusTransport:
    if sys.platform == "win32":
        pytest.skip("managed transport requires POSIX file locking")
    stream = tmp_path / "stdout.log"
    stream.write_text("")
    monkeypatch.setenv(ENV_STDOUT, str(stream))
    monkeypatch.setenv(ENV_BRIDGE, "1")
    monkeypatch.setattr(
        managed_dbus,
        "read_context",
        lambda timeout: LiveDocumentRef(window_id=WINDOW_A, document_id=DOC_A, name="same.svg"),
    )
    transport = ManagedDBusTransport(Settings())
    monkeypatch.setattr(
        transport, "get_selection", lambda: LiveSelection(object_ids=["box"], count=1)
    )
    return transport


def test_new_connection_requires_explicit_document_choice(bridge: ManagedDBusTransport) -> None:
    with pytest.raises(LiveError, match="choose the task drawing"):
        bridge.apply_to_selection(style={"fill": "red"}, transform=None)


@pytest.mark.parametrize("window,document", [(WINDOW_B, DOC_A), (WINDOW_A, DOC_B)])
def test_window_switch_or_revert_refuses_before_any_action(
    bridge: ManagedDBusTransport, monkeypatch: pytest.MonkeyPatch, window: str, document: str
) -> None:
    bridge.selected_document = LiveDocumentRef(window_id=window, document_id=document)
    monkeypatch.setattr(bridge, "_activate", lambda *args: pytest.fail("must not mutate"))
    with pytest.raises(LiveError, match="task drawing changed"):
        bridge.apply_to_selection(style={"fill": "blue"}, transform=None)


def test_native_activation_carries_verified_identity(
    bridge: ManagedDBusTransport, monkeypatch: pytest.MonkeyPatch
) -> None:
    calls: list[tuple] = []
    bridge.selected_document = LiveDocumentRef(window_id=WINDOW_A, document_id=DOC_A)
    monkeypatch.setattr(managed_dbus, "context_call", lambda *args: calls.append(args))
    result = bridge.apply_to_selection(style={"fill": "blue"}, transform=None)
    assert result.affected_ids == ["box"]
    assert calls == [("Activate", 60.0, WINDOW_A, DOC_A, "object-set-property", "[<'fill, blue'>]")]


def test_switch_during_selection_read_is_refused_by_native_activation(
    bridge: ManagedDBusTransport, monkeypatch: pytest.MonkeyPatch
) -> None:
    bridge.selected_document = LiveDocumentRef(window_id=WINDOW_A, document_id=DOC_A)

    def changed(*args: object) -> str:
        assert args[2:4] == (WINDOW_A, DOC_A)
        raise LiveError("document context unavailable or changed")

    monkeypatch.setattr(managed_dbus, "context_call", changed)
    with pytest.raises(LiveError, match="context unavailable or changed"):
        bridge.apply_to_selection(style={"fill": "blue"}, transform=None)
    # The failed call releases the operation lock and local identity.
    monkeypatch.setattr(managed_dbus, "context_call", lambda *args: "()")
    assert bridge.apply_to_selection(style={"fill": "green"}, transform=None).count == 1


def test_disconnect_discards_task_binding(bridge: ManagedDBusTransport) -> None:
    bridge.selected_document = LiveDocumentRef(window_id=WINDOW_A, document_id=DOC_A)
    bridge.disconnect()
    assert bridge.selected_document is None


def test_failed_selection_does_not_bind_another_window(
    bridge: ManagedDBusTransport, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setattr(managed_dbus, "context_call", lambda *args: "()")
    with pytest.raises(LiveError, match="activate the chosen drawing"):
        bridge.select_document(WINDOW_B, DOC_B)
    assert bridge.selected_document is None


def test_dropped_mutation_reply_is_uncertain(
    bridge: ManagedDBusTransport, monkeypatch: pytest.MonkeyPatch
) -> None:
    from inkscape_mcp.live.transport import LiveConnectionError, LiveMutationUncertain

    bridge.selected_document = LiveDocumentRef(window_id=WINDOW_A, document_id=DOC_A)

    def lost(*args: object) -> str:
        raise LiveConnectionError("reply unavailable")

    monkeypatch.setattr(managed_dbus, "context_call", lost)
    with pytest.raises(LiveMutationUncertain, match="inspect the task drawing"):
        bridge.apply_to_selection(style={"fill": "blue"}, transform=None)
