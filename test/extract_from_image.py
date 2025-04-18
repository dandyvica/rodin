"""extract artefacts from image using offsets"""
import sys


def extract_binary_segment(input_file, output_file, start_hex, end_hex):
    try:
        start = int(start_hex, 16)
        end = int(end_hex, 16)

        if start >= end:
            print("Start offset must be less than end offset.")
            return

        with open(input_file, "rb") as f:
            f.seek(start)
            data = f.read(end - start)

        with open(output_file, "wb") as out:
            out.write(data)

        print(
            f"Extracted {len(data)} bytes from {hex(start)} to {hex(end)} into '{output_file}'.")

    except Exception as e:
        print(f"Error: {e}")


if __name__ == "__main__":
    input_file = sys.argv[1]
    audit_file = sys.argv[2]

    count = 0
    for i, l in enumerate(open(audit_file)):
        tab = l.strip().split()
        if len(tab) != 4:
            continue

        offsets = tab[2].replace('(', '').replace(')', '').split('-')

        target_file = f"jpg_{count:08}.jpg"
        extract_binary_segment(input_file, target_file, offsets[0],  offsets[1])
        count += 1

