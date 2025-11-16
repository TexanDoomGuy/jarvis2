import textwrap
from PIL import Image, ImageDraw, ImageSequence, ImageFont
import io
import sys

text = " ".join(sys.argv[1:])

def wrap_text(text, font, max_width):
    """Wrap text into lines that fit within max_width (in pixels)."""
    wrapped_lines = []
    for raw_line in text.split("\n"):  # preserve manual line breaks
        # First split into rough words
        words = raw_line.split(" ")
        if not words:
            wrapped_lines.append("")  # empty line
            continue

        current_line = ""
        for word in words:
            test_line = word if current_line == "" else current_line + " " + word
            # Measure width
            w = font.getbbox(test_line)[2]
            if w <= max_width:
                current_line = test_line
            else:
                wrapped_lines.append(current_line)
                current_line = word
        wrapped_lines.append(current_line)
    return wrapped_lines


def draw_text_on_frames(text):
    font_size = 30
    im = Image.open("jarvis.gif")
    frames = []

    for frame in ImageSequence.Iterator(im):
        font = ImageFont.truetype("ARIAL.TTF", font_size)
        # --- Proper word wrapping ---
        wrapped_lines = wrap_text(text, font, frame.width - 20)
        new_height = frame.height + (len(wrapped_lines)*font_size)+20
        new_frame = Image.new("RGBA", (frame.width, new_height), (255, 255, 255, 255))
        new_frame.paste(frame, (0, (new_frame.height - frame.height) + 1))
        d = ImageDraw.Draw(new_frame)


        # Measure each wrapped line
        line_heights = []
        line_widths = []

        for line in wrapped_lines:
            bbox = font.getbbox(line)
            w = bbox[2] - bbox[0]
            h = bbox[3] - bbox[1]
            line_widths.append(w)
            line_heights.append(h)

        total_height = sum(line_heights)

        # vertically centered above the GIF
        y = ((new_frame.height - frame.height) / 2) - (total_height / 2)

        # Draw lines centered
        for i, line in enumerate(wrapped_lines):
            w = line_widths[i]
            h = line_heights[i]
            x = (frame.width - w) / 2
            d.text((x, y), line, font=font, fill=(0, 0, 0, 255))
            y += h

        del d

        # Prepare frame for GIF output
        b = io.BytesIO()
        new_frame.save(b, format="GIF")
        new_frame = Image.open(b)
        frames.append(new_frame)

    # Save final animation
    frames[0].save(
        "out.gif",
        save_all=True,
        append_images=frames[1:],
        loop=0,
        duration=100,
        optimize=False,
    )

draw_text_on_frames(text)
