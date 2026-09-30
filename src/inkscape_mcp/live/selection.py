"""Per-host backend probing, ranking, and selection.

The registration seam that lets (extension-socket) and (DBus) plug in without the
tool layer knowing either backend. ``probe_transports`` runs every registered backend's per-host
probe — returning the FULL set of available transports rather than one assumed by OS — and
``select_transport`` ranks the available ones (filtered to those that support the required
semantic commands) and returns the best-ranked instance.

The extension-socket bridge supplies the full read surface. The experimental macOS managed
session ranks above it to keep the GUI interactive while reading the current selection.
Ordinary DBus remains an action-only fast path. ``--app-id-tag`` is not used as a control API.
"""

from __future__ import annotations

from inkscape_mcp.config import Settings, get_settings
from inkscape_mcp.live.dbus_backend import DBusTransport
from inkscape_mcp.live.managed_dbus import ManagedDBusTransport
from inkscape_mcp.live.protocol import LiveCommand
from inkscape_mcp.live.socket_backend import ExtensionSocketTransport
from inkscape_mcp.live.transport import LiveTransport, TransportProbe

#: Registered backend classes (the plug-in seam). Order is irrelevant — ranking decides.
_BACKENDS: tuple[type[LiveTransport], ...] = (
    ExtensionSocketTransport,
    DBusTransport,
    ManagedDBusTransport,
)

#: The semantic read commands the live read tools require. A transport must support all of
#: these to be eligible for a read-mode connection; this is what keeps DBus (action-only) from
#: being selected for reads while still being reported as an available transport.
READ_REQUIRED: frozenset[LiveCommand] = frozenset(
    {
        LiveCommand.GET_ACTIVE_DOCUMENT,
        LiveCommand.GET_SELECTION,
        LiveCommand.INSPECT_SELECTION,
    }
)

#: Minimum command surface for a no-freeze connection: only the export-based active-document
#: read. A no-freeze connect also filters to transports whose `no_freeze` flag is set.
#: The managed macOS path additionally serves selection ids through captured process stdout;
#: ordinary DBus has no selection-id reads.
NO_FREEZE_REQUIRED: frozenset[LiveCommand] = frozenset({LiveCommand.GET_ACTIVE_DOCUMENT})


def probe_transports(settings: Settings | None = None) -> list[TransportProbe]:
    """Probe every registered backend on this host; return all results, ranked best-first.

    Each backend reports availability independently (no OS assumptions); a backend whose probe
    raises is reported as unavailable rather than crashing the whole probe.
    """
    s = settings if settings is not None else get_settings()
    probes: list[TransportProbe] = []
    for backend in _BACKENDS:
        try:
            probes.append(backend.probe(s))
        except Exception:  # pragma: no cover - a backend probe must never break detection
            probes.append(
                TransportProbe(
                    name=getattr(backend, "name", backend.__name__),
                    available=False,
                    rank=getattr(backend, "rank", 0),
                    supported_commands=[],
                    detail="probe failed",
                )
            )
    probes.sort(key=lambda p: (p.available, p.rank), reverse=True)
    return probes


def best_available(
    settings: Settings | None = None,
    required: frozenset[LiveCommand] | None = None,
    no_freeze: bool = False,
) -> TransportProbe | None:
    """Return the best-ranked AVAILABLE probe that supports `required`, or None.

    `required` defaults to the read surface (`READ_REQUIRED`); pass an empty set to pick the best
    available transport regardless of read capability (e.g. liveness only). When `no_freeze` is set,
    only transports that drive the GUI without freezing it are
    considered.
    """
    req = READ_REQUIRED if required is None else required
    for probe in probe_transports(settings):
        if not probe.available:
            continue
        if no_freeze and not probe.no_freeze:
            continue
        if req <= {LiveCommand(c) for c in probe.supported_commands if _is_command(c)}:
            return probe
    return None


def select_transport(
    settings: Settings | None = None,
    required: frozenset[LiveCommand] | None = None,
    no_freeze: bool = False,
) -> LiveTransport | None:
    """Instantiate the best available, capability-matching transport (NOT yet connected).

    Returns None when no available transport satisfies `required` (and, when `no_freeze` is set, is
    a no-freeze transport) — the clean "no live transport" path that the connect tool reports
    without treating it as an error.
    """
    s = settings if settings is not None else get_settings()
    chosen = best_available(s, required, no_freeze)
    if chosen is None:
        return None
    if chosen.name == ExtensionSocketTransport.name:
        return ExtensionSocketTransport(s)
    if chosen.name == DBusTransport.name:
        return DBusTransport(s)
    if chosen.name == ManagedDBusTransport.name:
        return ManagedDBusTransport(s)
    return None  # pragma: no cover - chosen always corresponds to a registered backend


def _is_command(value: str) -> bool:
    try:
        LiveCommand(value)
    except ValueError:
        return False
    return True
