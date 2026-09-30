"""Non-modal reads and selection edits for an Inkscape process we launched.

Inkscape's select-list GAction writes to the GUI process's stdout, not its D-Bus
reply. The managed launcher retains that stream. A following query-x action
prints a numeric fence, distinguishing an empty selection from an incomplete
reply. No clipboard, extension session, or GUI input simulation is involved.
"""

from __future__ import annotations

import os
import re
import sys
import time
from collections.abc import Iterator
from contextlib import contextmanager
from pathlib import Path
from threading import RLock
from typing import ClassVar

from inkscape_mcp.config import Settings
from inkscape_mcp.document.inspect import ObjectInfo, _bbox_of, _paint_of
from inkscape_mcp.live.dbus_backend import (
    DBusTransport,
    _variant_bool,
    _variant_empty,
    _variant_string,
)
from inkscape_mcp.live.protocol import LiveCommand
from inkscape_mcp.live.transport import (
    LiveConnectionError,
    LiveError,
    LiveMutationResult,
    LiveSelection,
    LiveSelectionInspection,
    TransportProbe,
)
from inkscape_mcp.workspace.xml_safety import parse_svg_bytes

ENV_STDOUT = "INKSCAPE_MCP_MANAGED_STDOUT"
_NUMBER = re.compile(r"^-?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?$")
_SELECTION_LINE = re.compile(r"^(\S+) cloned: (?:true|false) ref: \d+ href: \d+ total href: \d+$")
_LOCK = RLock()


def parse_selection_reply(text: str) -> list[str] | None:
    """None means no complete numeric fence yet; [] means a complete empty selection."""
    ids: list[str] = []
    for line in text.splitlines(keepends=True):
        if not line.endswith("\n"):
            return None
        line = line.strip()
        if _NUMBER.fullmatch(line):
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
    supported_commands: ClassVar[frozenset[LiveCommand]] = frozenset(
        {
            LiveCommand.PING,
            LiveCommand.GET_ACTIVE_DOCUMENT,
            LiveCommand.GET_DOCUMENT_SVG,
            LiveCommand.GET_SELECTION,
            LiveCommand.INSPECT_SELECTION,
            LiveCommand.RENDER_VIEW,
            LiveCommand.APPLY_TO_SELECTION,
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
        return probe

    @contextmanager
    def _operation(self) -> Iterator[Path]:
        import fcntl  # POSIX-only; this transport is advertised only on macOS.

        stream = os.environ.get(ENV_STDOUT)
        if not stream:
            raise LiveConnectionError("managed Inkscape stdout is not configured")
        path = Path(stream)
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
                yield path
            finally:
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
            return super()._export_document_bytes(fmt, **export_kwargs)

    def inspect_selection(self) -> LiveSelectionInspection:
        ids = set(self.get_selection().object_ids)
        root = parse_svg_bytes(self.get_document_svg().encode("utf-8")).getroot()
        objects = [
            ObjectInfo(
                id=elem.get("id"),
                tag=elem.tag.rsplit("}", 1)[-1],
                label=elem.get("{http://www.inkscape.org/namespaces/inkscape}label"),
                has_style=bool(elem.get("style") or elem.get("fill") or elem.get("stroke")),
                paint=_paint_of(elem),
                bbox=_bbox_of(elem),
            )
            for elem in root.iter()
            if isinstance(elem.tag, str) and elem.get("id") in ids
        ]
        return LiveSelectionInspection(objects=objects, count=len(objects))

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
