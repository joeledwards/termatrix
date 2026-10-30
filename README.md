# termatrix 3.0 — first rewrite milestone

Matrix rain with independent trace speeds, randomized depth and live configuration.
This is a new major-version implementation, ready for visual iteration, not a
release candidate.

```sh
cargo run -- --config ./preview.toml
```

From the rust-utilities workspace use:

```sh
scripts/with-rust cargo run --manifest-path repos/termatrix/Cargo.toml -- --config repos/termatrix/preview.toml
```

`?` toggles the top control panel. Up/down selects a setting; left/right adjusts
it. Space pauses simulation. q, Escape and Ctrl+C exit. Resize recomputes layout.
Very small terminals clip the controls; enlarge the terminal or hide the panel.
The panel occupies 12 rows (clipped to the terminal height), shows up to eight
settings at a time, and scrolls with selection. It may occupy more than half a
small terminal. Fresh configurations default to the `special` alphabet
(halfwidth Katakana); saved alphabet choices remain in effect.

Settings are loaded from the platform configuration directory's
`termatrix/profiles/default.toml`, or `--config` / `TERMATRIX_CONFIG`. In-app edits save
after 400ms without further changes and pending edits save on exit. Save failures
appear in the panel. Saves replace the file atomically using a sibling temporary
file. Comments are not preserved. External file changes are not watched; do not
edit the file concurrently with the application. Named presets are deferred.

Precedence is defaults, backing TOML, environment options, explicit CLI options,
then live edits. The next live save includes startup overrides. Missing files use
defaults; malformed or invalid files fail before entering raw mode. Parent
configuration directories are created on save. No file is created until an edit.

## Animation contract

- The library owns glyphs, traces, spatial bounds and seeded randomness. It has
  no terminal dependencies in its API. Orthographic projection supplies symbols,
  logical positions, appearance roles and trace identity to the frontend.
- A trace emits glyphs at integer head positions; earlier glyphs stay in place.
  Its deque is head-first, and its tail position is derived rather than stored.
- Simulation advances at fixed 10ms quanta. Rendering has an independent 1–240 Hz
  cap. Input activity does not change spawning. Stalls consume at most 250ms of
  simulation time per iteration; explicit pause freezes it.
- Speeds are abstract positive floating-point values. Compiled safety constants in `src/settings.rs` constrain
  speeds to `MIN_TRACE_SPEED = 0.01` through `MAX_TRACE_SPEED = 1000.0`.
  These are code-level limits, not panel or configuration settings. Bias -1 favors slow, 0 is uniform and +1 favors fast.
  Existing traces retain their sampled percentile when these settings change.
- Glyph-relative mode uses a reference advancement scale. Canvas-relative mode
  scales advancement by logical canvas height / 30. This first experiment keeps
  glyph spacing and length fixed: it compares traversal speeds, not perspective.
  Changing modes or resizing retains head positions and speed percentiles.
- Density targets active traces per logical column, capped at 2,048 traces and
  100,000 reserved glyph slots. Startup is empty. Population grows gradually;
  reducing density lets existing traces finish. Reserved slots bound growth.
- Clumping mixes uniform placement with selection near existing traces (within
  two lanes, including their lane). It affects new traces, not existing positions.
- Every trace has a fixed random depth. Smaller depth is nearer; larger IDs win
  equal-depth ties. Occluded traces continue evolving and reappear when uncovered.
  Only occupied glyph positions occlude; empty trace regions are transparent.
- Length changes affect newly spawned traces. Palette and color-mode changes
  recolor all existing glyphs immediately, including while paused. Alphabet
  changes also affect subsequent glyph emissions and mutations in existing traces.
  Mutation is a per-glyph time-based rate, independent of rendering.
- White bold heads and dim tails are fixed appearance rules. Selected body/tail
  glyphs are highlighted with bold and an explicitly brighter RGB shade.
  `highlight_probability` (0–1, default 0.12) adjusts this live; each glyph
  retains its random highlight percentile through mutation and palette changes. The
  weighted palette uses `bcgGkmrRwyos` (`o` is orange). Repeated letters
  increase their weight. Color choices are stable per trace, or per glyph with
  `per_glyph_colors = true` (the Individual glyph colors control). Head highlighting
  remains white in both modes. The compact panel list includes Green, Blue, Red, Purple, Yellow, Orange, Rainbow,
  Purple-green-blue, Orange-green-blue, Christmas (red/green/white), and
  White-gray. TOML accepts custom sequences. All named colors use explicit RGB. The shared `term-colors` dark palette
  provides red, orange, yellow, green, blue, purple and grey; local additions
  are cyan (51,187,204), white (238,238,238), and black (34,34,34).
- Terminal lanes allocate two cells for a Unicode scalar of width one or two.
  Grapheme clusters are not yet supported. Shrinking the canvas removes traces
  outside its columns or below its bottom. Showing the panel reduces canvas height.

## Migration from 2.x

`-c/--colors`, `-g/--glyph-set`, and `-f` remain. `--update-frequency` is an alias
for `--max-refresh-rate`; the environment name is now
`TERMATRIX_MAX_REFRESH_RATE`. Colors and alphabets retain their existing names.
`--min-step-delay` and `--max-step-delay` are replaced by abstract speed settings
in TOML/the live panel. `--seed` selects reproducible initial randomness (default
42); identical settings and simulation inputs reproduce runs.

This is an exclusively interactive animation. Redirected input/output and
`TERM=dumb` are rejected before terminal setup. It intentionally has no scrolling
fallback or machine output. Animation palette colors are artistic; panel text and
save errors carry meaning without depending on color. Ratatui owns frame diffing;
the scene is not reconstructed from the terminal. Normal exits, I/O errors and
panic unwinding restore raw mode, cursor visibility and the alternate screen.

## Development

```sh
cargo test --lib --bins
cargo fmt --check
cargo clippy --lib --bins -- -D warnings
```

See [coverage](docs/coverage.md) for tested behavior and outstanding checks.

## Control panel styling

The panel uses orange numeric values, yellow refresh rates and keys, a purple
title and directory paths, and green alphabet names. Toggles read On/Off, with
green/grey styling. Palette previews spell out their colors. Selection has a `>`
marker, bold label and subdued background. Save status reads Saved (green),
Pending (yellow), or Save failed (red). These text cues remain readable without
color. Selection, toggles and artistic palette previews are local UI conventions.

Yellow uses the shared dark-theme RGB (221,204,51) for both panel accents and
animation, avoiding theme-defined ANSI yellow that may appear brown.

The terminal frontend requires `term-colors` 0.2.0 (including `Style::swatch()`).
A local path supports workspace development; packaged builds resolve version 0.2
from crates.io. Publish term-colors 0.2.0 before packaging/publishing termatrix.
Ratatui receives RGB swatches directly; it does not
use the library's ANSI text formatter. This first frontend targets truecolor
terminals; automatic reduced-depth adaptation remains future work. The control
panel has explicit dark background and foreground colors.

Fresh settings default to `colors = "gG"` with individual glyph colors enabled:
light green is shared dark-theme green (102,187,51), and dark green is shared
light-theme green (51,136,0). Saved palettes and color-mode choices are preserved.
Select Green and enable Individual glyph colors in an existing session
to get the same mixed-shade effect. Highlight probability is adjustable in the
panel and persisted; zero disables random highlights while retaining white heads.

Older files containing `allowed_min_speed` / `allowed_max_speed` still load;
those obsolete entries are ignored and omitted on the next live save.

## Visible density chart

At widths of at least 112 columns (and panel heights of at least 7 rows), the
right side of the controls shows a Braille line chart. It measures occupied
logical glyph positions after clipping and occlusion, divided by the current
logical canvas area. Overlaps count once; a two-cell lane is one position.
The orange value is current coverage; the green line shows the last 60 seconds
on a fixed 0–100% scale. This differs from the Density population setting.

Sampling runs at up to 5 Hz independently of refresh, with at most 301 samples.
Stalls leave gaps rather than inventing historical samples. History continues
while the panel is hidden or simulation is paused (coverage remains flat while
paused unless layout changes). Resizes and panel toggles change the denominator;
this can create an honest discontinuity in measured coverage. Narrow terminals
show only controls. No extra charting dependency is required.

When the default profile is absent, the previous `termatrix/settings.toml` is
loaded as a migration fallback. The next live save writes `profiles/default.toml`
and leaves the legacy file untouched. An existing default profile always wins;
invalid or unreadable files produce an error rather than silently resetting.
Explicit `--config` / `TERMATRIX_CONFIG` paths bypass this legacy fallback.
Named-profile selection and management remain future work.

The White-gray profile uses `colors = "ws"`: white plus shared gray. Individual
glyph colors mixes shades within each trace; disabling it mixes shades across
traces. White head highlights and brighter random accents remain. Custom `w`
still means plain white; select White-gray to change an existing saved session.

Green mixes light/dark green (`gG`), Blue mixes blue/cyan (`bc`), and Red mixes
shared dark-theme red (221,68,51) with light-theme red (170,17,0), encoded as `rR`.
Purple (`m`), Yellow (`y`) and Orange (`o`) are single-hue profiles. Existing
custom sequences remain valid, including plain `g`.
