# FluentFlyout Rust Rewrite Plan

## Goal

Replace the current WPF/C# application with a Rust desktop application while keeping the visible design, layout, interaction timing, and Windows-native feel as close to the current build as possible.

This is not a direct port. The current app is built around WPF, XAML, `WPF-UI`, and `MicaWPF`, while the Rust code in this repository is currently limited to a partial native core in [`fluent_flyout_core`](/mnt/c/Users/minse/.gemini/antigravity/playground/spatial-kilonova/FluentFlyout/fluent_flyout_core).

## Non-Negotiables

- Match the current FluentFlyout design language.
- Keep the same top-level windows and settings surface.
- Preserve current feature behavior before removing the WPF app.
- Ship the Rust app only after side-by-side parity is acceptable.

## Current Inventory

- UI framework: WPF/XAML
- Main dependencies: `WPF-UI`, `MicaWPF`, `Dubya.WindowsMediaController`, `NAudio`
- Existing Rust scope: media/system interop DLL in [`fluent_flyout_core/Cargo.toml`](/mnt/c/Users/minse/.gemini/antigravity/playground/spatial-kilonova/FluentFlyout/fluent_flyout_core/Cargo.toml:1)
- XAML files: 46
- C# files: 148
- Main runtime windows/pages:
  - Media flyout window
  - Next Up window
  - Lock keys window
  - Taskbar window
  - Settings window with 8 navigation entries

Key source entry points:

- [`FluentFlyoutWPF/App.xaml`](/mnt/c/Users/minse/.gemini/antigravity/playground/spatial-kilonova/FluentFlyout/FluentFlyoutWPF/App.xaml:1)
- [`FluentFlyoutWPF/MainWindow.xaml`](/mnt/c/Users/minse/.gemini/antigravity/playground/spatial-kilonova/FluentFlyout/FluentFlyoutWPF/MainWindow.xaml:1)
- [`FluentFlyoutWPF/SettingsWindow.xaml`](/mnt/c/Users/minse/.gemini/antigravity/playground/spatial-kilonova/FluentFlyout/FluentFlyoutWPF/SettingsWindow.xaml:1)
- [`FluentFlyoutWPF/ViewModels/UserSettings.cs`](/mnt/c/Users/minse/.gemini/antigravity/playground/spatial-kilonova/FluentFlyout/FluentFlyoutWPF/ViewModels/UserSettings.cs:1)
- [`FluentFlyoutWPF/Classes/RustInterop.cs`](/mnt/c/Users/minse/.gemini/antigravity/playground/spatial-kilonova/FluentFlyout/FluentFlyoutWPF/Classes/RustInterop.cs:1)

## Dependency Port Map

The current C# application depends on several WPF-era packages. They do not all need the same migration treatment.

### Rebuild directly in Rust

These are part of the app's visible Windows design language or shell behavior. They must be recreated explicitly on the Rust side instead of being replaced with generic widgets.

- `WPF-UI`
  - Rebuild the visual primitives used by this app:
    - `NavigationView`
    - `TitleBar`
    - `CardAction`
    - `CardControl`
    - `InfoBar`
    - `Anchor`
    - toggle/select setting rows
  - This is the most important design-system port.
- `MicaWPF`
  - Recreate Mica, acrylic, and blur window/background treatment using Windows APIs from Rust.
  - This is required for parity with the current settings shell and flyout surfaces.
- `WPF-UI.Tray`
  - Recreate tray icon lifecycle, menus, click behavior, and icon theme switching in Rust.

### Replace with native Rust implementation

These are runtime or architecture helpers. They should not be ported 1:1. They should be replaced with Rust-native code that provides the same behavior.

- `CommunityToolkit.Mvvm`
  - Replace with Rust state structs, message flow, and explicit bindings.
  - No direct package-level port is needed.
- `Dubya.WindowsMediaController`
  - Replace with direct `windows` crate media-session integration.
  - This app already moved in that direction.
- `Microsoft.Toolkit.Uwp.Notifications`
  - Replace with direct Windows toast API integration from Rust.
- `NAudio`
  - Replace with Rust audio stack:
    - `cpal`
    - `rodio`
    - `windows` WASAPI bindings
    - `rustfft` where visualization/FFT is needed
- `NLog`
  - Replace with `tracing`, `log`, and `tracing-subscriber`.
- `System.Drawing.Common`
  - Replace with Rust imaging/rendering crates as needed:
    - `image`
    - `tiny-skia`
    - `resvg`
    - `skia-safe` if higher-fidelity raster/vector work becomes necessary

### Practical priority

For this rewrite, dependency work should be done in this order:

1. `WPF-UI`
2. `MicaWPF`
3. `WPF-UI.Tray`
4. `Dubya.WindowsMediaController` replacement cleanup
5. `Microsoft.Toolkit.Uwp.Notifications` replacement
6. `NAudio` replacement
7. logging and drawing replacements

### What this means in practice

The Rust rewrite should not start from app pages. It should start from a reusable component layer that mirrors the WPF design system used by the current app.

Recommended first Rust-side component set:

- `NavigationRail` / `NavigationView`
- `WindowTitleBar`
- `SettingsCardAction`
- `SettingsCardControl`
- `SettingsInfoBar`
- `AccentBadge`
- `AnchorButton`
- `TrayMenu`
- `MicaBackdropSurface`

Only after those exist should page-by-page porting continue.

## Target Stack

Recommended target:

- UI: Slint with Fluent style
- Windows integration: `windows` crate
- Domain/core logic: native Rust crates
- Packaging: MSIX or Windows desktop packaging after parity

Why this stack:

- Slint supports Windows desktop and Fluent styling.
- The existing Rust code already uses `windows`, so the system layer does not need a second foreign runtime.
- A webview stack would make pixel-level parity and native window feel harder, not easier.

## Migration Strategy

### Phase 0: Freeze the reference build

- Produce reference screenshots and recordings for all windows in light and dark themes.
- Record exact geometry, spacing, border radii, opacities, and animation durations.
- Snapshot current settings defaults and persisted settings behavior.

Exit criteria:

- Every major window has a reference capture set.
- Every user-facing setting is listed with type, default, and effect.

### Phase 1: Extract a parity specification

- Convert XAML layouts into an explicit UI spec.
- Map all resources from `App.xaml`, especially:
  - localization dictionaries
  - custom slider styling
  - custom tooltip styling
  - theme-bound brushes and fonts
- Define the Rust-side design tokens:
  - colors
  - spacing
  - radii
  - typography
  - motion timings

Exit criteria:

- A written parity checklist exists for each window.
- Shared design tokens are centralized instead of being inferred from XAML ad hoc.

### Phase 2: Move non-visual logic into Rust

- Keep WPF UI alive temporarily.
- Port these subsystems first:
  - settings load/save
  - media session querying and control
  - fullscreen detection
  - keyboard hooks
  - taskbar/widget geometry calculations
  - audio visualizer FFT
  - notifications and tray behavior

Exit criteria:

- WPF can call Rust for all core runtime logic.
- Existing FFI surface expands before it disappears.

### Phase 3: Build the Rust app shell

- Create a new Rust desktop binary as a sibling app, not a replacement in place.
- Recreate shell windows first:
  - media flyout shell
  - settings shell
  - lock keys shell
  - next up shell
  - taskbar shell
- Focus on geometry and styling before wiring all runtime behavior.

Exit criteria:

- All primary windows render and match reference captures closely.

### Phase 4: Rebuild the media flyout

- Port the current layout from `MainWindow.xaml`.
- Match:
  - album art block
  - song title and artist layout
  - control cluster
  - seekbar area
  - blurred background behavior
  - fly-in and fly-out motion

Exit criteria:

- Media flyout passes screenshot comparison and interaction smoke tests.

### Phase 5: Rebuild secondary runtime windows

- Port:
  - `NextUpWindow`
  - `LockWindow`
  - `TaskbarWindow`
  - taskbar widget and visualizer controls

Exit criteria:

- Secondary windows behave correctly across DPI and multi-monitor setups.

### Phase 6: Rebuild the settings experience

- Recreate the navigation window.
- Port all settings pages:
  - Home
  - Media Flyout
  - Taskbar Widget
  - Next Up
  - Lock Keys
  - Taskbar Visualizer
  - System
  - About

Exit criteria:

- Every setting currently exposed in WPF exists in Rust with equivalent behavior.

### Phase 7: Packaging and cutover

- Add app identity and packaging support.
- Rebuild installer or MSIX workflow for the Rust app.
- Run side-by-side test passes.
- Remove WPF only after the Rust app is the default shipping target.

Exit criteria:

- Rust build is the release artifact.
- WPF project is no longer required at runtime.

## Acceptance Criteria

The rewrite is complete only if all of the following are true:

- Visual parity is acceptable in light and dark mode.
- Startup, tray, media controls, and settings persistence work.
- Keyboard-triggered flyouts behave the same as the current app.
- Taskbar widget behavior is preserved.
- Localization still works.
- DPI scaling and multi-monitor behavior are validated.
- Packaging and update flow are restored.

## Immediate Implementation Order

1. Build the Rust UI shell and design token layer.
2. Move `UserSettings` into Rust data structures and persistence.
3. Expand `fluent_flyout_core` into a real reusable Rust domain crate.
4. Replace the media flyout first.
5. Replace the smaller windows.
6. Replace the settings window last.

## Risks

- Mica, blur, and transparency effects will not map 1:1 from WPF without explicit Windows-specific work.
- Text measurement and truncation may differ from WPF unless tuned carefully.
- Taskbar and tray behavior can regress if ported too early without visual parity locked first.
- A big-bang rewrite will likely stall. Side-by-side replacement is the safer path.

## Added Starting Point

The new Rust shell scaffold for this rewrite lives in:

- [`fluent_flyout_rs/Cargo.toml`](/mnt/c/Users/minse/.gemini/antigravity/playground/spatial-kilonova/FluentFlyout/fluent_flyout_rs/Cargo.toml:1)
- [`fluent_flyout_rs/src/main.rs`](/mnt/c/Users/minse/.gemini/antigravity/playground/spatial-kilonova/FluentFlyout/fluent_flyout_rs/src/main.rs:1)
- [`fluent_flyout_rs/ui/app.slint`](/mnt/c/Users/minse/.gemini/antigravity/playground/spatial-kilonova/FluentFlyout/fluent_flyout_rs/ui/app.slint:1)
