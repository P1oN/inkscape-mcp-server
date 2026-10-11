# Manager helper deadlines — 2026-10-11

The published v0.1.6 native acceptance encountered a helper stalled in File::open
on a regular update.lock in a Documents-based synthetic profile. Its sample was
captured about 88 seconds after launch. The cause remains unproven. The original
Manager already sent SIGTERM after 600 seconds; the observation did not reach or
invalidate that timer. Manual termination of only the owned helper produced the
Manager's recoverable interruption message.

Code inspection found three ways the existing timer could fail to release the UI:
a helper could ignore SIGTERM; a descendant could retain an output pipe after the
helper exited; or a request write on the main queue could block before the timer
was scheduled. The new implementation keeps the 600-second operation budget,
escalates to SIGKILL after two seconds, bounds pipe reads/writes by that deadline,
and writes requests off the main queue. Timeout invalidates the prepared UI state
and asks the user to Recover / retry. It does not bypass installer locking,
transaction recovery, path protections or signature verification.

A macOS Foundation subprocess fixture exercises the exact shared task-I/O code
with short test budgets: normal JSON completion; ignored SIGTERM; an exited helper
whose descendant holds stdout; oversized output; and a helper that never reads a
request exceeding pipe capacity. The Rust integration test compiles/runs this
fixture and independently limits every scenario to five seconds. The retained
output cap is one MiB and bounded progress-line processing is unchanged.

This is automated task-I/O acceptance, not native window acceptance or qualification
of new signed release bytes. It does not explain the Documents lock-file stall,
prove recovery from every interrupted transaction, or replace the other-Mac matrix.

Validation: Manager Objective-C syntax check with warnings treated as errors, tooling
fmt/Clippy (all targets), all 29 tooling tests, portable Clippy and git diff --check
pass locally. The five subprocess scenarios are included in the macOS tooling test
job. No MCP schema/instruction changes require manifest regeneration.

An authorized isolated native window fixture also passed. It compiled the current
Manager source with only the operation budget shortened from 600 to one second
(the two-second escalation grace was retained), and used a synthetic bundled helper
that ignored SIGTERM. The window showed “The operation timed out. Use Recover /
retry before trying again.” with its busy indicator off and controls enabled.
Recover / retry completed the same failure cycle; the idle window closed normally.
The app session exited with status zero and no process from the fixture directory
remained. Its HOME and installation were separate from the user's configuration.
This establishes native timeout presentation for the development fixture, not
acceptance of a new signed distribution or the original Documents-path cause.
