# FluentFlyout web UI shell

This folder is a static HTML/CSS/JavaScript prototype for the settings window.

- `index.html` owns the WPF-like shell: title bar, left navigation, and page host.
- `app.js` owns page data and shared card renderers.
- `styles.css` owns the Fluent/WPF-UI visual system.

Open `index.html` directly in a browser from the repository root. The shell uses the existing WPF preview images and Rust SVG icons through relative paths, so no asset copy step is required.
