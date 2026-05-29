import sys
import os

# Try to import Pillow safely
try:
    from PIL import Image
except ImportError:
    print("❌ Error: The 'pillow' library is missing.")
    print("💡 Fix: Install it using Homebrew by typing: brew install pillow")
    print("   Or bypass blocks with: pip3 install pillow --break-system-packages")
    sys.exit(1)

def convert_raw_to_png(raw_path, png_path, total_width, height):
    if not os.path.exists(raw_path):
        print(f"❌ Error: The file '{raw_path}' could not be found.")
        return

    try:
        with open(raw_path, "rb") as f:
            raw_bytes = f.read()
    except Exception as e:
        print(f"❌ Error reading raw file: {e}")
        return

    # Calculate byte-aligned stride (width rounded up to nearest multiple of 8)
    stride_bits = ((total_width + 7) // 8) * 8
    stride_bytes = stride_bits // 8
    expected_size = stride_bytes * height

    print(f"\n📂 Opening: {raw_path} ({len(raw_bytes)} bytes)")
    print(f"📐 Target Strip Size: {total_width}x{height} pixels")
    print(f"⚙️  Calculated Row Stride: {stride_bits} bits ({stride_bytes} bytes per row)")

    # Check if the file size matches the dimensions
    if len(raw_bytes) < expected_size:
        print(f"⚠️  Warning: File size is smaller than expected ({expected_size} bytes). Image might cut off.")

    # Create a new blank 1-bit monochrome image
    img = Image.new("1", (total_width, height))
    
    # Process bits row by row, skipping the invisible padding bits at the end of each stride row
    for y in range(height):
        row_start_byte = y * stride_bytes
        
        for x in range(total_width):
            byte_idx = row_start_byte + (x // 8)
            bit_idx = 7 - (x % 8)
            
            if byte_idx < len(raw_bytes):
                # Extract the single bit (0 or 1)
                bit = (raw_bytes[byte_idx] >> bit_idx) & 1
                # In PIL mode '1', 0 is black background, 255 is white text
                pixel_val = 255 if bit else 0
                img.putpixel((x, y), pixel_val)

    try:
        img.save(png_path)
        print(f"✅ Success! Saved long strip PNG to: {png_path}\n")
    except Exception as e:
        print(f"❌ Error saving PNG file: {e}")

if __name__ == "__main__":
    # Check if names were passed through the terminal command arguments
    if len(sys.argv) == 3:
        input_file = sys.argv[1]
        output_file = sys.argv[2]
        
        print("--- Target Dimensions Required ---")
        try:
            w = int(input("Enter TOTAL width of the long strip (e.g. 224): "))
            h = int(input("Enter HEIGHT of the font (e.g. 40): "))
        except ValueError:
            print("❌ Error: Dimensions must be whole numbers.")
            sys.exit(1)
    else:
        # Prompt user for everything if arguments are missing
        print("--- Embedded Graphics RAW to PNG Converter ---")
        input_file = input("Enter the input RAW file name/path: ").strip()
        output_file = input("Enter the desired output PNG file name/path: ").strip()
        try:
            w = int(input("Enter TOTAL width of the long strip (e.g. 224): "))
            h = int(input("Enter HEIGHT of the font (e.g. 40): "))
        except ValueError:
            print("❌ Error: Dimensions must be whole numbers.")
            sys.exit(1)

    if input_file and output_file and w > 0 and h > 0:
        convert_raw_to_png(input_file, output_file, w, h)
    else:
        print("❌ Error: Invalid inputs provided.")
