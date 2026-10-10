# Apple Developer ID intermediates

Public certificates from [Apple PKI](https://www.apple.com/certificateauthority/), retrieved 2026-10-10.
They are imported as intermediate certificates into the ephemeral signing keychain;
they are never installed as new trust roots or given custom trust overrides.
macOS must still validate the identity against its existing Apple trust anchors.

| File | DER SHA-256 | Expires (UTC) |
| --- | --- | --- |
| DeveloperIDCA.cer (G1) | `7afc9d01a62f03a2de9637936d4afe68090d2de18d03f29c88cfb0b1ba63587f` | 2027-02-01 22:12:15 |
| DeveloperIDG2CA.cer (G2) | `f16cd3c54c7f83cea4bf1a3e6a0819c8aaa8e4a1528fd144715f350643d2df3a` | 2031-09-17 00:00:00 |

Both generations are needed because a valid G1-issued Developer ID identity does
not chain through G2. GitHub’s runner image explicitly preinstalls G2; release
preparation supplies both and preserves/restores the runner’s user keychain search
list so the selected identity’s intermediate chain can be found.
