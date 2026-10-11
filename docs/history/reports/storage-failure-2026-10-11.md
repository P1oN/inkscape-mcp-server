# Isolated storage and process-failure acceptance — 2026-10-11

This is local CLI acceptance on macOS 27.0 (26A428), Apple Silicon, using the
published v0.1.6 Developer ID helper/bootstrap `ec46066ff9bb32b0`, Team `DN263AX69U`.
The final DMG hash is recorded in the v0.1.6 release ledger. A 768 MiB APFS sparse
image mounted outside Documents held only a synthetic installation. A separate
synthetic HOME/CODEX_HOME, managed skill and existing SVG workspace were supplied.
No user's installed runtime/client configuration or Inkscape window was changed.

## Actual ENOSPC and retry

The local signed baseline runtime `1a52d477e623f8d5` was installed with the published
instruction content `1bb8fc45fd410e26f83960e0ec958d807ac23b586bdec319b9d434148836a698`.
An exclusively created filler file on the size-limited image reached ENOSPC after
544,210,944 bytes. Preparing the published runtime returned exit 1 and structured
`No space left on device (os error 28)` while copying a Frameworks staging file.
Hashes of active.json, settings.json, bootstrap.json, the Codex configuration and
all managed-skill files were equal before/after the failed preparation.

While the volume was still full, its installed launcher completed actual JSON-RPC
initialize and tools/list (100 tools in the default exposed surface), then exited
zero on stdin EOF. Only the owned filler was removed. Two explicit recovery calls
succeeded; retry preparation/activation selected `ec46066ff9bb32b0` and retained
`1a52d477e623f8d5` as the previous runtime. This proves staging ENOSPC refusal,
preservation, launchability and retry on a real size-limited filesystem.

## Actual installer process termination

An initial filesystem-notification attempt observed the journal too late: activation
completed normally, so it supplies no interruption evidence. A subsequent bounded
one-millisecond journal poll captured the exact preparation ID after journal creation.
Only that owned installer was stopped, the journal/selector were captured, and the
process was killed with SIGKILL. Exit reported SIGKILL with the journal retained.
The selector had not changed before this stop. Two recovery calls succeeded,
restored the prior selector and its one journaled destination, and removed the journal.
This is one real pre-selector journal interruption, alongside the broader automated
mutation-boundary fixtures; it does not prove real termination at every boundary.

## Post-selector disk-full observation and limits

A further bounded poll stopped activation after observing the newer selector while
its journal still existed. The owned volume was filled to ENOSPC and the installer
resumed. It completed with exit zero and removed its journal: the fill did not force
the intended final metadata write failure. This is not final-write-failure acceptance.
The selected newer runtime also completed actual initialize/tools/list (100 tools)
while full. After removing only the filler, recovery returned changed=false.

The original synthetic SVG SHA-256 remained
`69ad36cc9f247f83691b3d2469f10d77687b435f1e393b84b1100cf0df4e1204`.
The test image was ejected normally. Detailed captured JSON, hashes, selectors and
exit observations are retained in the owned local `enospc` evidence directory.
No runtime source change was required. Broader post-selector write failure,
physical media loss/power failure, whole protected-CI job interruption and the
other-Mac/browser-quarantine/real-client matrix remain unqualified. These local
observations must not be generalized to those gates.
