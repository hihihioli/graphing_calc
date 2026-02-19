# Graphing Calculator UI Controls

## Overview
The graphing calculator features a separate native UI control window built with egui/eframe, providing modern cross-platform controls for Windows and Linux.

**Note for macOS users**: Due to threading limitations on macOS, the separate control window is not available. On macOS, only the main graphics window will appear. The graphing visualization will still work, but you won't have access to the UI controls for changing settings.

## Features

### Control Window
When you run the application, a separate control window will open with the following features:

#### View Settings
- **X Center**: Text field to set the X coordinate of the center of the view
- **Y Center**: Text field to set the Y coordinate of the center of the view
- **Zoom**: Text field to adjust the zoom level (higher values = more zoomed in)
- **Reset View**: Button to reset all view values to default (0, 0, 20)

#### Rendering Settings
- **Anti-Aliasing**: Dropdown menu to select the anti-aliasing mode:
  - **None**: No anti-aliasing (best performance)
  - **FXAA**: Fast Approximate Anti-Aliasing (good quality, minimal performance impact)
  - **TAA**: Temporal Anti-Aliasing (best quality, uses frame history for smoothest result)
- **Bloom Effect**: Toggle the bloom/glow effect on bright areas
- **Static Colors**: Toggle between animated color cycling and static colors

#### Function Expression
- **Function Expression**: Single-line text input for the fragment formula (GLSL syntax)
- **Apply**: Recompile the shader with the new expression
- **Reset**: Restore the default expression
- **Available variables**: `x`, `y`, `log_x`, `log_y`, `a_phase`, `zoom_phase`

#### Current Values Display
- Shows the real-time current X, Y, and Zoom values

### Main Graphics Window
- The main window displays the animated graphing visualization in a 1280x720 window
- You can drag with the mouse to pan around the view
- Mouse movements are synced with the control window
- Only the FPS counter is displayed as an overlay

## Usage

**Note**: The control window is not available on macOS due to platform threading limitations. On macOS, only the main graphics window will be displayed.

1. Run the application: `cargo run --release`
2. Two windows will appear (on Windows and Linux):
   - Main graphics window (1280x720, windowed)
   - Control window (separate, resizable)
3. On macOS, only the main graphics window will appear
4. Use the control window (Windows/Linux) to:
   - Enter precise coordinates and zoom values in the text fields
   - Toggle rendering effects with checkboxes
   - Reset the view to defaults
4. Or use mouse dragging in the main window to pan around
5. All changes are synchronized between both windows in real-time

## Technical Details
- Built with eframe/egui for native UI
- Thread-safe communication using Arc<Mutex<GraphParams>>
- Supports Windows and Linux natively (macOS support limited to main graphics window only)
- Modern, GPU-accelerated UI rendering
- Settings are now managed through the UI instead of keyboard hotkeys (on supported platforms)
