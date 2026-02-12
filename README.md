# Graphing Calculator

A high-performance graphing calculator with real-time post-processing effects, built with Rust and macroquad.

## Features

### Real-Time Graphing
- Visualizes mathematical functions with exponential zoom
- Red and blue color-coded values (positive/negative)
- Parallel rendering for optimal performance
- Dynamic zoom controls

### Post-Processing Effects

1. **Vignette** (Default: ON)
   - Darkens edges for cinematic focus
   - Pre-computed lookup table for efficiency

2. **Glow/Bloom**
   - Creates luminous halo around bright areas
   - Gaussian blur with configurable radius
   - Toggle with `1` key

3. **Chromatic Aberration**
   - Simulates lens color fringing
   - Subtle red/blue channel offsets
   - Toggle with `2` key

4. **Scanlines**
   - Retro CRT monitor effect
   - Horizontal scan overlay
   - Toggle with `3` key

5. **Swirl Distortion** (NEW!)
   - Slow rotating distortion effect
   - Intensity decreases from center to edges
   - Toggle with `5` key

### Interactive Controls

#### Effect Toggles
- `1` - Toggle Glow/Bloom effect
- `2` - Toggle Chromatic Aberration
- `3` - Toggle Scanlines
- `4` - Toggle Vignette
- `5` - Toggle Swirl Distortion
- `D` - Toggle Debug Menu

#### Movement Controls
- `M` - Switch between Auto-Zoom and Manual mode
- **Auto-Zoom Mode** (default): Continuous automatic zoom
- **Manual Mode**:
  - `↑` (Up Arrow) - Zoom In
  - `↓` (Down Arrow) - Zoom Out

### Debug Menu

Press `D` to show/hide the interactive debug menu featuring:
- Real-time effect status indicators
- Color-coded ON/OFF states (GREEN/GRAY)
- Movement mode display (YELLOW/ORANGE)
- Keyboard shortcuts reference
- FPS counter

## Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run
cargo run --release
```

## Requirements

- Rust 2024 edition
- macroquad 0.4.14
- rayon 1.8

## Performance

The application uses CPU-based rendering with Rayon parallelization:
- Base graph rendering is fully parallelized
- Effects are optional and independently toggleable
- Pre-computed lookup tables where possible
- Efficient memory access patterns

### Typical Performance
- 60+ FPS on modern hardware (fullscreen)
- All effects enabled: 30-60 FPS depending on resolution
- Individual effects: negligible performance impact

## Technical Details

### Architecture
- **Framework**: macroquad (cross-platform game framework)
- **Parallelization**: Rayon for multi-threaded rendering
- **Effect Pipeline**: Sequential CPU-based post-processing

### Effect Processing Order
1. Base image generation (parallel)
2. Vignette (if enabled)
3. Glow/bloom (if enabled)
4. Swirl distortion (if enabled)
5. Texture update
6. Base texture rendering
7. Chromatic aberration overlay (if enabled)
8. Scanlines overlay (if enabled)
9. Debug menu UI (if enabled)
10. FPS counter

### Safety Features
- Comprehensive bounds checking on all array accesses
- Validated screen dimensions before rendering
- Protected zoom controls (minimum scale bounds)
- No unsafe code

## Known Limitations

- GPU compute shaders not currently supported (macroquad limitation)
- Fullscreen only
- Fixed mathematical function (can be modified in source)

## Future Enhancements

Potential improvements for future versions:
- GPU compute shader support for effects
- Custom function input
- Effect intensity sliders
- Save/load effect presets
- Screenshot capability
- Additional distortion effects

## License

See repository license file.

## Contributing

Feel free to submit issues and enhancement requests!
