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
from typing import ClassVar
from uuid import uuid4

from inkscape_mcp.config import Settings
from inkscape_mcp.live.dbus_backend import (
    DBusTransport,
    _variant_bool,
    _variant_empty,
    _variant_string,
)
from inkscape_mcp.live.insert_payload import document_fingerprint, prepare_fragment
from inkscape_mcp.live.managed_scene import object_info, scene_from_svg
from inkscape_mcp.live.protocol import LiveCommand
from inkscape_mcp.live.transport import (
    LiveCapabilityUnsupported,
    LiveConnectionError,
    LiveError,
    LiveMutationResult,
    LiveScene,
    LiveSelection,
    LiveSelectionInspection,
    TransportProbe,
)
from inkscape_mcp.workspace.subprocess_exec import ProcessError, run_process
from inkscape_mcp.workspace.xml_safety import parse_svg_bytes

ENV_STDOUT = "INKSCAPE_MCP_MANAGED_STDOUT"
ENV_DIR = "INKSCAPE_MCP_MANAGED_DIR"
INSERT_ACTION = "org.inkscape-mcp.insert.noprefs"
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
        }
    )

    @classmethod
    def probe(cls, settings: Settings) -> TransportProbe:
        probe = super().probe(settings)
        stream = os.environ.get(ENV_STDOUT)
        probe.supported_commands = [c.value for c in sorted(cls.supported_commands)]
        if sys.platform != "darwin" or not stream or not Path(stream).is_file():
            probe.available = False
            probe.detail = "start Inkscape with inkscape-mcp-macos to capture its selection replies"
        elif probe.available:
            probe.detail = (
                "managed Inkscape: current selection and undoable edits, no modal session"
            )
        if not cls._insert_available(settings.process_timeout_s):
            probe.supported_commands.remove(LiveCommand.INSERT_SVG.value)
        return probe

    @classmethod
    def _insert_available(cls, timeout: float) -> bool:
        if not os.environ.get(ENV_DIR):
            return False
        try:
            result = run_process(
                cls._actions_call_argv("Describe", INSERT_ACTION), timeout_s=timeout
            )
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
        return super().supports(command)

    @contextmanager
    def _operation(self) -> Iterator[Path]:
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
                yield path
            finally:
                del _LOCAL.path
                fcntl.flock(lock, fcntl.LOCK_UN)

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
            return scene_from_svg(self.get_document_svg(), selected)

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
                self._activate(INSERT_ACTION, _variant_empty())
                deadline = time.monotonic() + self._settings.process_timeout_s
                while time.monotonic() < deadline:
                    if reply.is_file():
                        if reply.stat().st_size > 1024 * 1024:
                            raise LiveError("insertion reply exceeds size cap")
                        result = json.loads(reply.read_text())
                        if result.get("nonce") != nonce or not result.get("ok"):
                            raise LiveError("Inkscape refused insertion; document context changed")
                        if nonce in self.get_document_svg():
                            return LiveMutationResult(
                                affected_ids=ids,
                                count=len(ids),
                                detail="inserted one SVG group; one native Undo step",
                                undo_friendly=True,
                            )
                    time.sleep(0.05)
                raise LiveConnectionError(
                    "insertion did not complete; inspect Inkscape before retrying"
                )
            finally:
                request.unlink(missing_ok=True)
                reply.unlink(missing_ok=True)

    def apply_to_selection(
        self, *, style: dict[str, str], transform: str | None
    ) -> LiveMutationResult:
        # This first milestone intentionally supports one fill edit = one native Undo
        # entry. Do not pretend several separate GActions form an atomic transaction.
        if set(style) != {"fill"} or transform is not None:
            raise LiveError("managed prototype supports a single fill change only")
        selected = self.get_selection()
        if not selected.object_ids:
            raise LiveError("select an object in the managed Inkscape window first")
        with self._operation():
            result = super().apply_to_selection(style=style, transform=None)
        result.affected_ids = selected.object_ids
        result.count = selected.count
        result.detail = "changed fill on the current selection; use Inkscape Undo to revert"
        return result
