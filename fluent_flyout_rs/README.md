# fluent_flyout_rs

Rust desktop runtime for the FluentFlyout rewrite.

Current scope:

- Creates a parallel Rust entry point instead of replacing the WPF app in place.
- Uses Slint with Fluent styling for the media flyout and lock flyout surfaces.
- Reads `%AppData%/FluentFlyout/settings.xml` and applies the current media flyout layout rules.
- Reads and controls the active Windows media session, including seek, repeat, shuffle, and album art.
- Uses the existing Rust core as an in-process library for keyboard hook and fullscreen detection.
- Shows the media flyout from media and volume keys and shows a lock flyout for Caps/Num/Scroll/Insert.
- Adds a minimal tray icon with `Open Dashboard`, `Show Media Flyout`, and `Quit`.

Run from Windows or WSL with the Windows Rust toolchain:

```bash
/mnt/c/Users/minse/.cargo/bin/cargo.exe run --manifest-path fluent_flyout_rs/Cargo.toml
```

If the local Rust toolchain is unavailable, install it first or run via an existing Windows-side Rust setup.
