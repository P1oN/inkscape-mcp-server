#!/usr/bin/env python3
"""Check bounded codesign display output, after cryptographic signature verification."""
import sys


def validate(info, native):
    lines = info.splitlines()
    if not any(line.startswith('Authority=Developer ID Application:') for line in lines):
        raise ValueError('signature is not Developer ID Application')
    timestamps = [line.removeprefix('Timestamp=') for line in lines if line.startswith('Timestamp=')]
    if len(timestamps) != 1 or not timestamps[0].strip() or timestamps[0] == 'none':
        raise ValueError('signature is missing a secure timestamp')
    if native and '(runtime)' not in info:
        raise ValueError('native code is missing hardened runtime')


if __name__ == '__main__':
    try:
        if len(sys.argv) != 2 or sys.argv[1] not in ('native', 'container'):
            raise ValueError('Usage: verify-signature-info.py native|container < CODESIGN_DISPLAY')
        data = sys.stdin.buffer.read(65537)
        if len(data) > 65536:
            raise ValueError('signature display exceeds byte budget')
        validate(data.decode('utf-8'), sys.argv[1] == 'native')
    except (ValueError, UnicodeError) as error:
        sys.exit(str(error))
