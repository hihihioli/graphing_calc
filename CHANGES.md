# Changes Summary

## Bug Fixes

### 1. Fixed Index Out of Bounds Panic
- **Issue**: Potential panic at runtime due to accessing array indices without bounds checking
- **Solution**: Added comprehensive bounds checking throughout the codebase:
  - Added `if idx < img_data.len()` checks in `apply_glow()` function
  - Added bounds checking in vignette application loop
  - Added bounds checking in all post-processing effect loops
  - Added bounds checking in new `apply_swirl()` function

### 2. Fixed Chromatic Aberration Effect
- **Issue**: Chromatic aberration was making everything black and purple by drawing multiple full white textures on top of each other
- **Solution**: Completely rewrote the chromatic aberration implementation:
  - Now draws the base texture first in full color
  - Then adds subtle red and blue channel offsets (0.15 intensity instead of full brightness)
  - Uses small 2-pixel offset instead of percentage-based offset
  - Creates proper color fringing effect without washing out the image

## New Features

### 1. Swirl Distortion Effect
- **Key**: Press `5` to toggle
- **Description**: Implements a slow, rotating swirl distortion effect
- **Details**:
  - Swirl intensity decreases from center to edges for a natural look
  - Slow rotation speed (0.5x time multiplier) for pleasant visual effect
  - Proper bounds checking to prevent crashes
  - Integrated into the post-processing pipeline

### 2. Debug Menu / UI Panel
- **Key**: Press `D` to toggle menu visibility
- **Features**:
  - Semi-transparent black background for readability
  - Shows all available effects with their current state (ON/OFF)
  - Color-coded status indicators:
    - GREEN = Effect is enabled
    - GRAY = Effect is disabled
    - YELLOW/ORANGE = Movement mode indicators
    - SKYBLUE = Manual control hints
  - Displays all keyboard shortcuts
  - Shows FPS counter
  - Clean, organized layout

### 3. Movement Mode Toggle
- **Key**: Press `M` to toggle between AUTO-ZOOM and MANUAL modes
- **AUTO-ZOOM Mode** (default):
  - Automatically zooms in continuously
  - Same behavior as original implementation
- **MANUAL Mode**:
  - Arrow Up: Zoom in
  - Arrow Down: Zoom out
  - Gives user full control over zoom level
  - Can pause zoom at any level

## Technical Improvements

### 1. Safety Enhancements
- Added comprehensive bounds checking to all array accesses
- Prevents potential panics from out-of-bounds access
- Validates indices before accessing `img_data` and `vignette_lut`

### 2. Code Organization
- Added time tracking variable for time-based effects
- Improved variable naming and structure
- Better separation of concerns in the rendering pipeline
- More maintainable effect toggle system

### 3. Performance Considerations
- Effects are optional and can be toggled independently
- Bounds checking has minimal performance impact
- Swirl effect only runs when enabled
- Debug menu rendering is conditional

## Effect Order in Pipeline

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

## Controls Summary

| Key | Effect |
|-----|--------|
| 1 | Toggle Glow/Bloom |
| 2 | Toggle Chromatic Aberration |
| 3 | Toggle Scanlines |
| 4 | Toggle Vignette |
| 5 | Toggle Swirl Distortion |
| D | Toggle Debug Menu |
| M | Toggle Movement Mode (Auto-zoom vs Manual) |
| Up Arrow | Zoom In (Manual mode only) |
| Down Arrow | Zoom Out (Manual mode only) |

## GPU Compute Shader Note

The current implementation uses CPU-based post-processing with Rayon for parallelization. While GPU compute shaders would provide better performance for some effects, macroquad's current architecture doesn't provide direct compute shader support. The effects are optimized for CPU execution with:
- Parallel processing using Rayon for base image generation
- Efficient lookups (pre-computed vignette LUT)
- Optional effects to reduce overhead when not needed
- Minimal allocations in the hot path

For future GPU acceleration, consider:
- Using WGPU compute shaders directly
- Implementing effects as fragment shaders
- Using a different framework with better GPU compute support
