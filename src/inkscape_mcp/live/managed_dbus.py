"""Non-modal reads and selection edits for an Inkscape process we launched.

Inkscape's select-list GAction writes to the GUI process's stdout, not its D-Bus
reply. The managed launcher retains that stream. A following query-x action
prints a numeric fence, distinguishing an empty selection from an incomplete
reply. No clipboard, extension session, or GUI input simulation is involved.
"""

from __future__ import annotations

import json
import os
import re
import sys
import time
from collections.abc import Iterator
from contextlib import contextmanager
from pathlib import Path
from threading import RLock, local
from typing import Any, ClassVar
from uuid import uuid4

from inkscape_mcp.config import Settings
from inkscape_mcp.live.context_bridge import (
    ENV_BRIDGE,
    context_call,
    list_contexts,
    read_context,
    validate_identity,
)
from inkscape_mcp.live.dbus_backend import (
    DBusTransport,
    _variant_bool,
    _variant_empty,
    _variant_string,
)
from inkscape_mcp.live.edit_errors import EDIT_REFUSALS
from inkscape_mcp.live.geometry import root_mapping
from inkscape_mcp.live.insert_payload import document_fingerprint, prepare_fragment
from inkscape_mcp.live.managed_scene import document_ref, object_info, scene_from_svg
from inkscape_mcp.live.protocol import LiveCommand
from inkscape_mcp.live.transport import (
    LiveCapabilityUnsupported,
    LiveConnectionError,
    LiveContextError,
    LiveDocumentRef,
    LiveEditRefused,
    LiveError,
    LiveMutationResult,
    LiveMutationUncertain,
    LiveScene,
    LiveSelection,
    LiveSelectionInspection,
    RenderRegion,
    TransportProbe,
)
from inkscape_mcp.workspace.subprocess_exec import ProcessError, run_process
from inkscape_mcp.workspace.xml_safety import UnsafeXMLError, parse_svg_bytes

ENV_STDOUT = "INKSCAPE_MCP_MANAGED_STDOUT"
ENV_DIR = "INKSCAPE_MCP_MANAGED_DIR"
INSERT_ACTION = "org.inkscape-mcp.insert.noprefs"
EDIT_ACTION = "org.inkscape-mcp.edit.noprefs"
_NUMBER = re.compile(r"^-?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?$")
_SELECTION_LINE = re.compile(r"^(\S+) cloned: (?:true|false) ref: \d+ href: \d+ total href: \d+$")
_LOCK = RLock()
_LOCAL = local()


def parse_selection_reply(text: str) -> list[str] | None:
    """None means no complete numeric fence yet; [] means a complete empty selection."""
    ids: list[str] = []
    for line in text.splitlines(keepends=True):
        if not line.endswith("\n"):
            return None
        line = line.strip()
        if all(_NUMBER.fullmatch(part) for part in line.split(",")):
            return list(dict.fromkeys(ids))
        match = _SELECTION_LINE.fullmatch(line)
        if match:
            ids.append(match[1])
        elif line:
            # Do not silently turn a changed Inkscape output format into an empty selection.
            raise LiveError("unexpected selection response from managed Inkscape")
    return None


class ManagedDBusTransport(DBusTransport):
    """D-Bus actions plus bounded stdout reads, verified on macOS Inkscape 1.4.3."""

    name: ClassVar[str] = "managed-dbus"
    rank: ClassVar[int] = 30
    _root_id: str | None = None
    selected_document: LiveDocumentRef | None = None

    @property
    def context_guard_available(self) -> bool:
        return os.environ.get(ENV_BRIDGE) == "1"

    def list_documents(self) -> list[LiveDocumentRef]:
        if not self.context_guard_available:
            raise LiveCapabilityUnsupported(
                "restart a managed session with the context bridge after saving your work"
            )
        return list_contexts(self._settings.process_timeout_s)

    def select_document(self, window_id: str, document_id: str) -> LiveDocumentRef:
        validate_identity(window_id, document_id)
        if not self.context_guard_available:
            raise LiveCapabilityUnsupported("document context bridge is unavailable")
        with self._operation():
            context_call("SelectDocument", self._settings.process_timeout_s, window_id, document_id)
            deadline = time.monotonic() + min(self._settings.process_timeout_s, 2.0)
            while True:
                current = read_context(self._settings.process_timeout_s)
                if (current.window_id, current.document_id) == (window_id, document_id):
                    self.selected_document = current
                    return current
                if time.monotonic() >= deadline:
                    raise LiveContextError(
                        "activate the chosen drawing and call live_select_document again"
                    )
                time.sleep(0.02)

    def _require_selected(self) -> None:
        if not self.context_guard_available:
            return  # Compatibility with already running pre-bridge sessions.
        current = getattr(_LOCAL, "context", None)
        selected = self.selected_document
        if selected is None:
            raise LiveContextError(
                "choose the task drawing with live_select_document before editing"
            )
        if current is None or (current.window_id, current.document_id) != (
            selected.window_id,
            selected.document_id,
        ):
            raise LiveContextError("task drawing changed; call live_select_document before editing")

    def _activate(
        self, action: str, parameter: str, *, object_path: str = "/org/inkscape/Inkscape"
    ) -> None:
        try:
            context = getattr(_LOCAL, "context", None)
            if self.context_guard_available:
                if context is None:
                    raise LiveError("context action requires a managed operation")
                context_call(
                    "Activate",
                    self._settings.process_timeout_s,
                    context.window_id,
                    context.document_id,
                    action,
                    parameter,
                )
            else:
                super()._activate(action, parameter, object_path=object_path)

        except LiveConnectionError as exc:
            if action in {"object-set-property", INSERT_ACTION, EDIT_ACTION}:
                raise LiveMutationUncertain(
                    "edit completion uncertain; inspect the task drawing before retrying"
                ) from exc
            raise

    def disconnect(self) -> None:
        self.selected_document = None
        super().disconnect()

    supported_commands: ClassVar[frozenset[LiveCommand]] = frozenset(
        {
            LiveCommand.PING,
            LiveCommand.GET_ACTIVE_DOCUMENT,
            LiveCommand.GET_DOCUMENT_SVG,
            LiveCommand.GET_SELECTION,
            LiveCommand.INSPECT_SELECTION,
            LiveCommand.RENDER_VIEW,
            LiveCommand.APPLY_TO_SELECTION,
            LiveCommand.GET_SCENE,
            LiveCommand.INSERT_SVG,
            LiveCommand.SET_SELECTED_TEXT,
        }
    )

    @classmethod
    def probe(cls, settings: Settings) -> TransportProbe:
        probe = super().probe(settings)
        stream = os.environ.get(ENV_STDOUT)
        probe.supported_commands = [c.value for c in sorted(cls.supported_commands)]
        if sys.platform != "darwin" or not stream or not Path(stream).is_file():
            probe.available = False
            probe.detail = (
                "explicitly open Inkscape with live_launch or inkscape-mcp-macos --launch"
            )
        elif probe.available:
            probe.detail = (
                "managed Inkscape: current selection and undoable edits, no modal session"
            )
        if not probe.available or not cls._insert_available(settings.process_timeout_s):
            probe.supported_commands.remove(LiveCommand.INSERT_SVG.value)
        if not probe.available or not cls._effect_available(
            settings.process_timeout_s, EDIT_ACTION
        ):
            probe.supported_commands.remove(LiveCommand.SET_SELECTED_TEXT.value)
        return probe

    @classmethod
    def _insert_available(cls, timeout: float) -> bool:
        return cls._effect_available(timeout, INSERT_ACTION)

    @classmethod
    def _effect_available(cls, timeout: float, action: str) -> bool:
        if not os.environ.get(ENV_DIR):
            return False
        try:
            result = run_process(cls._actions_call_argv("Describe", action), timeout_s=timeout)
            return (
                result.returncode == 0
                and not result.timed_out
                and result.stdout.strip().startswith("((true,")
            )
        except (ProcessError, LiveError):
            return False

    def supports(self, command: LiveCommand) -> bool:
        if command == LiveCommand.INSERT_SVG:
            return self._insert_available(self._settings.process_timeout_s)
        if command == LiveCommand.SET_SELECTED_TEXT:
            return self._effect_available(self._settings.process_timeout_s, EDIT_ACTION)
        return super().supports(command)

    def is_connected(self) -> bool:
        """Do not report a vanished managed bus as connected from a cached flag."""
        return self._connected and self._actions_list_reachable(
            min(self._settings.process_timeout_s, 2.0)
        )

    def get_active_document(self) -> LiveDocumentRef:
        try:
            with self._operation():
                root = parse_svg_bytes(self.get_document_svg().encode()).getroot()
                ref = document_ref(root)
                context = getattr(_LOCAL, "context", None)
                if context is not None:
                    ref.window_id = context.window_id
                    ref.document_id = context.document_id
        except UnsafeXMLError as exc:
            raise LiveError("active document export is invalid") from exc
        except OSError as exc:
            raise LiveError("active document export is unavailable") from exc
        return ref

    @contextmanager
    def _operation(self) -> Iterator[Path]:
        if sys.platform == "win32":
            raise LiveConnectionError("managed session requires a POSIX platform")
        import fcntl  # POSIX-only; this transport is advertised only on macOS.

        stream = os.environ.get(ENV_STDOUT)
        if not stream:
            raise LiveConnectionError("managed Inkscape stdout is not configured")
        path = Path(stream)
        with _LOCK:
            if getattr(_LOCAL, "path", None) is not None:
                if _LOCAL.path != path:
                    raise LiveError("cannot nest operations from different managed sessions")
                yield path
                return
        # Coordinate all MCP clients reusing the same managed GUI process.
        with _LOCK, path.with_suffix(".lock").open("a") as lock:
            deadline = time.monotonic() + self._settings.process_timeout_s
            while True:
                try:
                    fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
                    break
                except BlockingIOError:
                    if time.monotonic() >= deadline:
                        raise LiveConnectionError("managed Inkscape is busy") from None
                    time.sleep(0.02)
            try:
                _LOCAL.path = path
                _LOCAL.context = (
                    read_context(self._settings.process_timeout_s)
                    if self.context_guard_available
                    else None
                )
                yield path
            finally:
                if hasattr(_LOCAL, "context"):
                    del _LOCAL.context
                del _LOCAL.path
                fcntl.flock(lock, fcntl.LOCK_UN)

    @contextmanager
    def operation_scope(self) -> Iterator[None]:
        """Pin document identity across context capture, previews, and the edit."""
        with self._operation():
            yield

    def get_selection(self) -> LiveSelection:
        with self._operation() as path, path.open("rb") as stream:
            stream.seek(0, os.SEEK_END)
            self._activate("select-list", _variant_empty())
            self._activate("query-x", _variant_empty())
            deadline = time.monotonic() + min(self._settings.process_timeout_s, 5.0)
            data = b""
            while time.monotonic() < deadline:
                data += stream.read(65536)
                if len(data) > 1024 * 1024:
                    raise LiveError("managed selection response exceeds the size cap")
                try:
                    ids = parse_selection_reply(data.decode("utf-8"))
                except UnicodeDecodeError:
                    ids = None  # a multibyte id may straddle the current read
                if ids is not None:
                    ids = [oid for oid in ids if oid != self._root_id]
                    return LiveSelection(object_ids=ids, count=len(ids))
                time.sleep(0.01)
            raise LiveConnectionError("managed Inkscape did not complete its selection reply")

    def _export_document_bytes(self, fmt: str, **export_kwargs: object) -> bytes:
        with self._operation():
            # Export settings persist in the GUI. Reset selection/text/plain-SVG options
            # rather than inheriting a previous export or stripping Inkscape layer metadata.
            self._activate("export-id", _variant_string(""))
            self._activate("export-id-only", _variant_bool(False))
            self._activate("export-text-to-path", _variant_bool(False))
            self._activate("export-plain-svg", _variant_bool(False))
            export_kwargs["plain_svg"] = False
            data = super()._export_document_bytes(fmt, **export_kwargs)
            if fmt == "svg":
                self._root_id = parse_svg_bytes(data).getroot().get("id")
            return data

    def inspect_selection(self) -> LiveSelectionInspection:
        with self._operation():
            ids = set(self.get_selection().object_ids)
            root = parse_svg_bytes(self.get_document_svg().encode("utf-8")).getroot()
            objects = [
                object_info(elem)
                for elem in root.iter()
                if isinstance(elem.tag, str) and elem.get("id") in ids
            ]
            return LiveSelectionInspection(objects=objects, count=len(objects))

    def get_scene(self) -> LiveScene:
        with self._operation():
            selected = self.get_selection().object_ids
            scene = scene_from_svg(self.get_document_svg(), selected)
            context = getattr(_LOCAL, "context", None)
            if context is not None and scene.active_document is not None:
                scene.active_document.window_id = context.window_id
                scene.active_document.document_id = context.document_id
            return scene

    def render_view(self, region: RenderRegion | None = None, scale: float | None = None) -> bytes:
        """Map document user units to the pixel coordinates required by export-area."""
        with self._operation():
            if region is not None:
                try:
                    root = parse_svg_bytes(self.get_document_svg().encode()).getroot()
                except UnsafeXMLError as exc:
                    raise LiveError("render region document could not be parsed safely") from exc
                sx, sy, tx, ty = root_mapping(root)
                region = RenderRegion(
                    x=region.x * sx + tx,
                    y=region.y * sy + ty,
                    width=region.width * sx,
                    height=region.height * sy,
                )
            return super().render_view(region=region, scale=scale)

    def insert_svg(self, svg_fragment: str) -> LiveMutationResult:
        nonce = "mcp_" + uuid4().hex
        try:
            _, ids = prepare_fragment(svg_fragment, nonce)
        except (ValueError, TypeError) as exc:
            raise LiveError("unsupported or unsafe SVG insertion fragment") from exc
        if not self.supports(LiveCommand.INSERT_SVG):
            raise LiveCapabilityUnsupported("restart managed Inkscape to load the insertion helper")
        directory = os.environ.get(ENV_DIR)
        if not directory:
            raise LiveConnectionError("managed insertion directory is unavailable")
        root = Path(directory)
        request = root / "insert-request.json"
        reply = root / "insert-result.json"
        with self._operation():
            self._require_selected()
            doc = parse_svg_bytes(self.get_document_svg().encode()).getroot()
            expected = [e.get("id") for e in doc.iter() if e.get("id")]
            reply.unlink(missing_ok=True)
            pending = root / "insert-request.tmp"
            pending.write_text(
                json.dumps(
                    {
                        "nonce": nonce,
                        "fragment": svg_fragment,
                        "expected_ids": expected,
                        "expected_fingerprint": document_fingerprint(doc),
                    }
                )
            )
            pending.chmod(0o600)
            pending.replace(request)
            try:
                try:
                    self._activate(INSERT_ACTION, _variant_empty())
                except LiveConnectionError as exc:
                    # A native error dialog can outlive the D-Bus call. The helper may
                    # already have reported refusal; reconcile that reply below.
                    if not reply.is_file():
                        raise LiveMutationUncertain(
                            "insertion activation did not complete; "
                            "inspect Inkscape before retrying"
                        ) from exc
                deadline = time.monotonic() + self._settings.process_timeout_s
                while time.monotonic() < deadline:
                    if reply.is_file():
                        if reply.stat().st_size > 1024 * 1024:
                            raise LiveError("insertion reply exceeds size cap")
                        try:
                            result = json.loads(reply.read_text())
                        except (ValueError, OSError) as exc:
                            raise LiveError(
                                "invalid insertion reply; inspect Inkscape before retrying"
                            ) from exc
                        if not isinstance(result, dict):
                            raise LiveError(
                                "invalid insertion reply; inspect Inkscape before retrying"
                            )
                        if result.get("nonce") != nonce or not result.get("ok"):
                            raise LiveError("Inkscape refused insertion; document context changed")
                        try:
                            applied = nonce in self.get_document_svg()
                        except (LiveError, OSError, UnsafeXMLError) as exc:
                            raise LiveMutationUncertain(
                                "insertion may have applied before the window changed; "
                                "inspect the selected task drawing before retrying"
                            ) from exc
                        if applied:
                            return LiveMutationResult(
                                affected_ids=ids,
                                count=len(ids),
                                detail="inserted one SVG group; one native Undo step",
                                undo_friendly=True,
                            )
                    time.sleep(0.05)
                raise LiveMutationUncertain(
                    "insertion did not complete; inspect Inkscape before retrying"
                )
            finally:
                request.unlink(missing_ok=True)
                reply.unlink(missing_ok=True)

    def _edit_selection(self, operation: str, **params: object) -> LiveMutationResult:
        if not self._effect_available(self._settings.process_timeout_s, EDIT_ACTION):
            raise LiveCapabilityUnsupported(
                "save and restart managed Inkscape to load everyday edits"
            )
        directory = os.environ.get(ENV_DIR)
        if not directory:
            raise LiveConnectionError("managed edit directory is unavailable")
        root = Path(directory)
        request, reply = root / "insert-request.json", root / "insert-result.json"
        nonce = "mcp_" + uuid4().hex
        with self._operation():
            self._require_selected()
            selected = self.get_selection()
            if not selected.object_ids:
                raise LiveError("select an object in the managed Inkscape window first")
            doc = parse_svg_bytes(self.get_document_svg().encode()).getroot()
            payload = {
                "nonce": nonce,
                "operation": operation,
                "selection": selected.object_ids,
                "expected_ids": [e.get("id") for e in doc.iter() if e.get("id")],
                "expected_fingerprint": document_fingerprint(doc),
                **params,
            }
            pending = root / "insert-request.tmp"
            activated = False
            failed = False
            try:
                reply.unlink(missing_ok=True)
                pending.write_text(json.dumps(payload))
                pending.chmod(0o600)
                pending.replace(request)
                try:
                    activated = True
                    self._activate(EDIT_ACTION, _variant_empty())
                except LiveConnectionError as exc:
                    if not reply.is_file():
                        raise LiveMutationUncertain(
                            "edit activation did not complete; "
                            "inspect the task drawing before retrying"
                        ) from exc
                deadline = time.monotonic() + self._settings.process_timeout_s
                while time.monotonic() < deadline:
                    if reply.is_file():
                        result = self._read_edit_reply(reply, nonce)
                        if not result["ok"]:
                            reason = result.get("error", "")
                            if reason not in EDIT_REFUSALS:
                                reason = "invalid document or selection"
                            raise LiveEditRefused("Inkscape refused edit: " + reason)
                        try:
                            current = parse_svg_bytes(self.get_document_svg().encode()).getroot()
                        except (LiveError, OSError, UnsafeXMLError) as exc:
                            raise LiveMutationUncertain(
                                "edit may have applied; inspect the task drawing before retrying"
                            ) from exc
                        if document_fingerprint(current) == result["fingerprint"]:
                            return LiveMutationResult(
                                affected_ids=result["ids"],
                                count=len(result["ids"]),
                                detail=f"{operation}: one Undo step; unchanged calls add no step",
                                undo_friendly=True,
                            )
                    time.sleep(0.05)
                raise LiveMutationUncertain(
                    "edit result not confirmed; inspect the task drawing before retrying"
                )
            except OSError as exc:
                failed = True
                if activated:
                    raise LiveMutationUncertain(
                        "edit exchange failed; inspect the task drawing before retrying"
                    ) from exc
                raise LiveError("could not prepare managed edit request") from exc
            except BaseException:
                failed = True
                raise
            finally:
                cleanup_failed = False
                for path in (request, reply, pending):
                    try:
                        path.unlink(missing_ok=True)
                    except OSError:
                        cleanup_failed = True
                if cleanup_failed and not failed:
                    raise LiveMutationUncertain(
                        "edit cleanup failed; inspect the task drawing before retrying"
                    )

    @staticmethod
    def _read_edit_reply(reply: Path, nonce: str) -> dict[str, Any]:
        try:
            if reply.stat().st_size > 1024 * 1024:
                raise ValueError("oversized reply")
            result = json.loads(reply.read_text())
            if not isinstance(result, dict) or result.get("nonce") != nonce:
                raise ValueError("stale reply")
            if type(result.get("ok")) is not bool:
                raise ValueError("invalid discriminator")
            if result["ok"] and (
                not isinstance(result.get("fingerprint"), str)
                or not isinstance(result.get("ids"), list)
                or any(not isinstance(oid, str) for oid in result["ids"])
            ):
                raise ValueError("invalid result")
            if not result["ok"] and not isinstance(result.get("error", ""), str):
                raise ValueError("invalid refusal")
        except (OSError, ValueError, TypeError) as exc:
            raise LiveMutationUncertain(
                "invalid edit reply; inspect the task drawing before retrying"
            ) from exc
        return result

    def apply_to_selection(
        self, *, style: dict[str, str], transform: str | None
    ) -> LiveMutationResult:
        if self._effect_available(self._settings.process_timeout_s, EDIT_ACTION):
            return self._edit_selection("style", style=style, transform=transform)
        # Preserve fill support in an already running pre-stage-3 GUI.
        if set(style) != {"fill"} or transform is not None:
            raise LiveCapabilityUnsupported(
                "save and restart managed Inkscape to load everyday edits"
            )
        with self._operation():
            self._require_selected()
            selected = self.get_selection()
            if not selected.object_ids:
                raise LiveError("select an object in the managed Inkscape window first")
            result = super().apply_to_selection(style=style, transform=None)
        result.affected_ids = selected.object_ids
        result.count = selected.count
        result.detail = "changed fill on the current selection; use Inkscape Undo to revert"
        return result

    def set_selected_text(self, text: str) -> LiveMutationResult:
        return self._edit_selection("text", text=text)

    def edit_selection(self, operation: str) -> LiveMutationResult:
        if operation not in {
            "duplicate",
            "delete",
            "group",
            "ungroup",
            "raise",
            "lower",
            "front",
            "back",
        }:
            raise LiveError("unsupported selection operation")
        return self._edit_selection(operation)
