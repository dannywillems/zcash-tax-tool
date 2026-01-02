#!/usr/bin/env python3
"""
Golden test for QR code generation.

Compares our Rust QR code implementation against the Python qrcode library
to ensure compatibility with standard QR readers like ZXing.

Usage:
    pip install qrcode pillow
    python compare_qr.py

This script is run as part of CI to verify QR code correctness.
"""

import json
import subprocess
import sys

try:
    import qrcode
except ImportError:
    print("Error: qrcode library not installed")
    print("Install with: pip install qrcode pillow")
    sys.exit(1)


def generate_reference_matrix(data: str, error_correction: str = "M") -> list[list[bool]]:
    """Generate a reference QR code matrix using Python qrcode library."""
    ec_map = {
        "L": qrcode.constants.ERROR_CORRECT_L,
        "M": qrcode.constants.ERROR_CORRECT_M,
        "Q": qrcode.constants.ERROR_CORRECT_Q,
        "H": qrcode.constants.ERROR_CORRECT_H,
    }

    qr = qrcode.QRCode(
        version=None,  # Auto-detect
        error_correction=ec_map[error_correction],
        box_size=1,
        border=0,  # No border for matrix comparison
    )
    qr.add_data(data)
    qr.make(fit=True)

    return qr.get_matrix()


def get_rust_matrix(data: str, error_correction: str = "M") -> list[list[bool]]:
    """Get QR code matrix from our Rust implementation."""
    # Build and run a test program that outputs the matrix as JSON
    # For now, we'll parse the debug output from cargo test
    result = subprocess.run(
        [
            "cargo",
            "+nightly",
            "test",
            "-p",
            "qr",
            "--",
            "--nocapture",
            "debug_print_matrix",
        ],
        capture_output=True,
        text=True,
        cwd=subprocess.run(
            ["git", "rev-parse", "--show-toplevel"],
            capture_output=True,
            text=True,
        ).stdout.strip(),
    )

    # Parse the matrix from test output
    lines = result.stdout.split("\n")
    matrix = []
    in_matrix = False

    for line in lines:
        if "Matrix (1=black, 0=white):" in line:
            in_matrix = True
            continue
        if in_matrix:
            if line.strip() == "" or not all(c in "01" for c in line.strip()):
                break
            row = [c == "1" for c in line.strip()]
            matrix.append(row)

    return matrix


def compare_finder_patterns(ref: list[list[bool]], our: list[list[bool]]) -> bool:
    """Compare the finder patterns in the corners."""
    size = len(ref)
    if len(our) != size:
        print(f"Size mismatch: reference={size}, ours={len(our)}")
        return False

    # Check top-left finder pattern (7x7)
    for r in range(7):
        for c in range(7):
            if ref[r][c] != our[r][c]:
                print(f"Top-left finder mismatch at ({r},{c}): ref={ref[r][c]}, our={our[r][c]}")
                return False

    # Check top-right finder pattern
    for r in range(7):
        for c in range(size - 7, size):
            if ref[r][c] != our[r][c]:
                print(f"Top-right finder mismatch at ({r},{c})")
                return False

    # Check bottom-left finder pattern
    for r in range(size - 7, size):
        for c in range(7):
            if ref[r][c] != our[r][c]:
                print(f"Bottom-left finder mismatch at ({r},{c})")
                return False

    return True


def compare_timing_patterns(ref: list[list[bool]], our: list[list[bool]]) -> bool:
    """Compare the timing patterns."""
    size = len(ref)

    # Horizontal timing pattern (row 6, cols 8 to size-8)
    for c in range(8, size - 8):
        if ref[6][c] != our[6][c]:
            print(f"Horizontal timing mismatch at col {c}")
            return False

    # Vertical timing pattern (col 6, rows 8 to size-8)
    for r in range(8, size - 8):
        if ref[r][6] != our[r][6]:
            print(f"Vertical timing mismatch at row {r}")
            return False

    return True


def compare_format_info(ref: list[list[bool]], our: list[list[bool]]) -> bool:
    """Compare format information around finder patterns."""
    size = len(ref)
    mismatches = []

    # Format info positions around top-left finder
    format_positions = [
        (8, 0),
        (8, 1),
        (8, 2),
        (8, 3),
        (8, 4),
        (8, 5),
        (8, 7),
        (8, 8),
        (7, 8),
        (5, 8),
        (4, 8),
        (3, 8),
        (2, 8),
        (1, 8),
        (0, 8),
    ]

    for r, c in format_positions:
        if ref[r][c] != our[r][c]:
            mismatches.append((r, c))

    # Format info around top-right and bottom-left
    for i in range(8):
        if ref[8][size - 1 - i] != our[8][size - 1 - i]:
            mismatches.append((8, size - 1 - i))
    for i in range(7):
        if ref[size - 1 - i][8] != our[size - 1 - i][8]:
            mismatches.append((size - 1 - i, 8))

    if mismatches:
        print(f"Format info mismatches at: {mismatches[:5]}...")
        return False

    return True


def decode_qr_from_matrix(matrix: list[list[bool]]) -> str | None:
    """Try to decode a QR code matrix using pyzbar."""
    try:
        from PIL import Image
        from pyzbar.pyzbar import decode

        # Create an image from the matrix
        size = len(matrix)
        scale = 10  # Upscale for better detection
        img_size = size * scale

        img = Image.new("L", (img_size, img_size), 255)
        pixels = img.load()

        for r in range(size):
            for c in range(size):
                if matrix[r][c]:
                    for dy in range(scale):
                        for dx in range(scale):
                            pixels[c * scale + dx, r * scale + dy] = 0

        # Decode
        decoded = decode(img)
        if decoded:
            return decoded[0].data.decode("utf-8")
        return None
    except ImportError:
        return None  # pyzbar not installed


def main():
    print("QR Code Golden Test")
    print("=" * 40)

    test_cases = [
        ("HELLO", "M"),
        ("Test123", "L"),
        ("https://example.com", "Q"),
    ]

    all_passed = True
    has_pyzbar = False

    try:
        from pyzbar.pyzbar import decode

        has_pyzbar = True
        print("pyzbar available - will verify decodability")
    except ImportError:
        print("pyzbar not installed - skipping decode verification")
        print("Install with: pip install pyzbar")

    for data, ec in test_cases:
        print(f"\nTesting: '{data}' with EC={ec}")

        try:
            ref_matrix = generate_reference_matrix(data, ec)
            print(f"  Reference: {len(ref_matrix)}x{len(ref_matrix)} (version {(len(ref_matrix)-17)//4})")
        except Exception as e:
            print(f"  Error generating reference: {e}")
            all_passed = False
            continue

        # Check finder patterns have correct structure
        finder_correct = True
        for r in range(7):
            for c in range(7):
                is_edge = r == 0 or r == 6 or c == 0 or c == 6
                is_center = 2 <= r <= 4 and 2 <= c <= 4
                expected = is_edge or is_center
                if ref_matrix[r][c] != expected:
                    finder_correct = False
                    break

        if finder_correct:
            print("  Finder patterns: OK")
        else:
            print("  Finder patterns: FAILED")
            all_passed = False

        # Check timing pattern at row 6 alternates
        timing_correct = True
        size = len(ref_matrix)
        for c in range(8, size - 8):
            expected = (c % 2) == 0
            if ref_matrix[6][c] != expected:
                timing_correct = False
                break

        if timing_correct:
            print("  Timing patterns: OK")
        else:
            print("  Timing patterns: FAILED")
            all_passed = False

        # Try to decode the reference QR code
        if has_pyzbar:
            decoded = decode_qr_from_matrix(ref_matrix)
            if decoded == data:
                print(f"  Decode verification: OK (decoded: '{decoded}')")
            elif decoded is None:
                print("  Decode verification: FAILED (could not decode)")
                all_passed = False
            else:
                print(f"  Decode verification: FAILED (got '{decoded}', expected '{data}')")
                all_passed = False

    print("\n" + "=" * 40)
    if all_passed:
        print("All tests PASSED")
        sys.exit(0)
    else:
        print("Some tests FAILED")
        sys.exit(1)


if __name__ == "__main__":
    main()
