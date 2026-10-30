# Changelog

## 3.1.0

- Add a `p` profile picker with live switching and creation from current settings,
  with distinct colors for keys, descriptions, names, creation and errors.
- Show the configuration profile name instead of its full path, retaining save feedback.
- Render tails with explicit darker RGB colors rather than terminal-dependent dim styling.
- Give remaining panel width to performance, with trace count, actual FPS and
  frame work time; prioritize frame-work, then coverage, then trace-count plots as space allows.

Existing 3.0 settings and command-line options remain compatible. Profile selection
applies during the current session; startup still uses the default or `--config`.

## 3.0.0

A complete rewrite with a frontend-independent animation model.

- Live controls (`?`) with automatically saved preferences.
- Resizable layout and a Braille chart of visible glyph coverage, with ten minutes
  of history and live, persisted logarithmic time compression.
- Independent trace speeds, depth occlusion, density, clumping and lengths.
- Explicit RGB palettes, two-shade color profiles and per-glyph highlights.
- Retained alphabet selection, weighted custom colors and startup overrides.
- Separate simulation timing and display refresh rate.
- Retry failed preference saves on exit and reset terminal styling during cleanup.

### Migration

- Min/Max speed replace the old millisecond step-delay options.
- `--max-refresh-rate` replaces `--update-frequency` (kept as an alias).
- The refresh environment variable is now `TERMATRIX_MAX_REFRESH_RATE`.
- Preferences live under `termatrix/profiles/default.toml` in the platform config directory.

See [the user guide](docs/guide.md) for details. Named-profile management is deferred.
