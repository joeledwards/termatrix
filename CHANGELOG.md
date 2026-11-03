# Changelog

## 3.3.1

- Tune fresh defaults to density 2.0, clumping 0.8 and maximum trace length 40.
  Saved preferences continue to take precedence.
- Match palette-manager outlines and headings to the profile picker in purple.
- Refresh README screenshots for the rain, controls, profiles and palettes.

## 3.3.0

- Wrap controls-panel command hints to available width so pause and quit remain
  visible; adjust the settings window for additional help rows.

- Add a `c` palette manager with named custom weighted color-code palettes, live
  previews, editing and confirmed deletion.
- Show/hide built-in and custom palettes when cycling; keep at least one visible.
- Persist catalog and visibility in shared `palettes.toml`, independently of
  profile color codes, with atomic saves and visible failure messages.

Existing profiles and CLI color codes remain compatible. Catalog edits and
deletions do not change already saved profile colors.

## 3.2.0

- Fade traces from head through primary color toward black or white, preserving the terminal background, with light/dark theme
  selection and automatic RGB, 256-color or ANSI approximation.
- Keep the final glyph faintly visible instead of fading fully to black or white.
- Cache reduced-color fade conversions across frames instead of searching the ANSI
  palette for every glyph; preserve exact colors and the truecolor fast path.
- Tune fresh defaults: density 1.7, clumping 0.7, maximum trace length 32,
  canvas-relative speed on, and highlight probability 2%; keep controls visible.
- Simplify individual palette previews to a single primary-colored name.
- Remove cyan from the built-in Rainbow palette and color mixed palette names
  directly, including Rainbow and Christmas, instead of appending color labels.

Saved settings remain compatible and override fresh defaults. Existing custom
palettes containing cyan remain supported.

## 3.1.1

- Refresh the main screenshot and add a profile-picker screenshot.
- Add a README Profiles section describing profile selection and creation.

Runtime behavior is unchanged from 3.1.0.

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
