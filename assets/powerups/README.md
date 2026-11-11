# Procedural Power-up Pixel Art

Each power-up folder contains:

- `<kind>.ron`: editable source data with `schema_version`, `kind`, `[32, 32]` grid, an 8-color palette, and variable animation frames. Each frame stores a duration and 32 indexed pixel-map rows. `.` means transparent; `0`–`7` index the palette.
- `frame_<n>.png`: generated 32×32 transparent runtime texture consumed by Bevy.

The RON files are the source maps; PNGs are generated runtime outputs. From the project root, regenerate with:

```bash
python3 tools/generate_powerup_pixel_art.py
python3 tools/validate_powerup_pixel_art.py
```

The ten asset keys are `speed`, `slow`, `enlarge`, `double_ball`, `fireball`, `snowball`, `reversal`, `sticky`, `ball_storm`, and `boomerang`.
