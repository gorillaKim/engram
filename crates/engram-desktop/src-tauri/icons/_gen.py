"""Generate a simple Engram app icon (1024x1024 PNG)."""
from PIL import Image, ImageDraw

SIZE = 1024
RADIUS = int(SIZE * 0.225)  # macOS Big Sur squircle corner ratio (~22.5%)

# Color palette — Indigo 600
BG = (79, 70, 229, 255)   # #4F46E5
FG = (255, 255, 255, 255)
ACCENT = (165, 180, 252, 255)  # Indigo 300 (subtle)

img = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
draw = ImageDraw.Draw(img)

# Rounded square background
draw.rounded_rectangle((0, 0, SIZE, SIZE), radius=RADIUS, fill=BG)

# Three horizontal bars (sprint → epic → issue hierarchy).
# Centered vertically, left-aligned with a node dot on the left of each.
bar_h = int(SIZE * 0.085)
gap = int(SIZE * 0.075)
total_h = bar_h * 3 + gap * 2
top = (SIZE - total_h) // 2

# Bar widths: longest top, shorter bottom (suggests hierarchy)
widths = [0.62, 0.50, 0.38]   # fractions of SIZE
left_pad = int(SIZE * 0.20)
dot_r = int(bar_h * 0.34)

for i, wf in enumerate(widths):
    y = top + i * (bar_h + gap)
    # Node dot on the left
    cx = left_pad
    cy = y + bar_h // 2
    draw.ellipse((cx - dot_r, cy - dot_r, cx + dot_r, cy + dot_r), fill=FG)
    # Bar to the right of the dot
    bar_left = cx + dot_r + int(SIZE * 0.025)
    bar_right = bar_left + int(SIZE * wf)
    draw.rounded_rectangle(
        (bar_left, y, bar_right, y + bar_h),
        radius=bar_h // 2,
        fill=FG if i < 2 else ACCENT,
    )

# Vertical connector line between dots (subtle accent)
connector_w = int(SIZE * 0.012)
cx = left_pad
top_dot_y = top + bar_h // 2
bottom_dot_y = top + 2 * (bar_h + gap) + bar_h // 2
draw.rectangle(
    (cx - connector_w // 2, top_dot_y, cx + connector_w // 2, bottom_dot_y),
    fill=ACCENT,
)

img.save("/tmp/engram-icon-source.png", "PNG")
print("Generated /tmp/engram-icon-source.png", img.size)
