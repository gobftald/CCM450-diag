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

def convert_png_to_raw(png_path, raw_path):
    if not os.path.exists(png_path):
        print(f"❌ Error: The file '{png_path}' could not be found.")
        return

    try:
        # Convert image to strict black and white (1-bit)
        img = Image.open(png_path).convert("1")
    except Exception as e:
        print(f"❌ Error reading image: {e}")
        return

    width, height = img.size
    
    # Calculate byte-aligned stride (width rounded up to nearest multiple of 8)
    stride = ((width + 7) // 8) * 8
    padding_bits = stride - width
    
    print(f"\n📂 Opening: {png_path}")
    print(f"📐 Image Dimensions: {width}x{height} pixels")
    print(f"⚙️  Row Stride: {stride} bits ({stride // 8} bytes per row)")
    if padding_bits > 0:
        print(f"🔗 Padding: Adding {padding_bits} blank bits to the end of each row.")

    raw_bytes = bytearray()

    # Pack pixels row by row
    for y in range(height):
        current_byte = 0
        bit_count = 0
        
        for x in range(width):
            pixel_val = img.getpixel((x, y))
            # 1 for lit pixel (white), 0 for unlit (black)
            bit = 1 if pixel_val > 0 else 0
            
            # Pack bit into byte from MSB to LSB
            current_byte = (current_byte << 1) | bit
            bit_count += 1
            
            if bit_count == 8:
                raw_bytes.append(current_byte)
                current_byte = 0
                bit_count = 0
                
        # Handle trailing row bits
        if bit_count > 0:
            current_byte <<= (8 - bit_count)
            raw_bytes.append(current_byte)

    try:
        with open(raw_path, "wb") as f:
            f.write(raw_bytes)
        print(f"✅ Success! Saved raw binary file to: {raw_path} ({len(raw_bytes)} bytes)\n")
        print(f"💡 For Rust: Use an image width stride of {stride} in ImageRaw::new().")
    except Exception as e:
        print(f"❌ Error saving raw file: {e}")

if __name__ == "__main__":
    # Check if names were passed through the terminal command arguments
    if len(sys.argv) == 3:
        input_file = sys.argv[1]
        output_file = sys.argv[2]
    else:
        # Prompt user if arguments are missing
        print("--- Embedded Graphics RAW Converter ---")
        input_file = input("Enter the input PNG file name/path: ").strip()
        output_file = input("Enter the desired output RAW file name/path: ").strip()

    if input_file and output_file:
        convert_png_to_raw(input_file, output_file)
    else:
        print("❌ Error: File names cannot be blank.")
