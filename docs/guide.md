# User guide

## Live settings

Press `?` to toggle the 12-row panel. Use up/down to select and left/right to
adjust. Selection scrolls through the settings. In very small terminals, the panel
is clipped to the available space. Space freezes the animation; q, Esc, or Ctrl+C
restores the terminal and exits.

| Setting | Meaning |
| --- | --- |
| Min / Max speed | Abstract trace advancement speeds, independent of display refresh |
| Speed bias | -1 favors slow traces; 0 is uniform; +1 favors fast traces |
| Density | Target active traces per logical column, including overlapping traces |
| Clumping | 0 spreads arrivals uniformly; 1 favors positions near existing traces |
| Min / Max length | Length range in glyphs, applied to new traces |
| Mutation rate | Average character changes per glyph per second; 0 freezes characters |
| Refresh cap | Maximum display frequency (1–240 Hz), independent of animation speed |
| Canvas-relative | On scales advancement with canvas height; Off uses glyph-relative speed |
| Palette | Built-in colors or the custom sequence loaded from TOML |
| Alphabet | Katakana, letters, numbers, symbols, and other character sets |
| Individual glyph colors | Off (default) chooses colors per trace; On chooses per glyph |
| Highlight probability | Chance a glyph receives a stable bright/bold accent; 0 disables accents |

Heads remain white. Hidden traces continue moving and can reappear. Startup is
empty; coverage grows as traces enter from the top. The chart measures visible
coverage after occlusion, not the population target controlled by Density.
It samples at 5 Hz, retains 60 seconds, and counts overlapping glyphs once.
History continues while paused or hidden. Resizing changes the measured area.

## Saved preferences

Default locations use the platform configuration directory:

- Linux: `$XDG_CONFIG_HOME/termatrix/profiles/default.toml`, usually under `~/.config`.
- macOS: `~/Library/Application Support/termatrix/profiles/default.toml`.
- Windows: `%APPDATA%\termatrix\config\profiles\default.toml`.

`--config <path>` or `TERMATRIX_CONFIG` overrides the location. Changes save after
400 ms without further edits, with pending changes saved on exit. Save failures
appear in the panel. Failed saves remain unsaved and are retried on exit;
if the retry fails, the app exits with an error after restoring terminal state.
Saving replaces the file atomically and creates parent
directories. TOML comments are not preserved. Do not edit the file externally
while the app is running; external changes are not watched.

Precedence: defaults, saved file, environment options, explicit CLI options,
then live changes. The next save includes startup overrides. Missing files use
defaults; invalid files report an error before terminal setup.

An older `termatrix/settings.toml` loads only if the default profile is absent;
the next edit saves to the new profile location and leaves the old file untouched.
Explicit paths bypass this fallback. Obsolete `allowed_min_speed` and
`allowed_max_speed` keys are ignored and omitted on save. Actual hard speed limits
are code constants in `src/settings.rs`: 0.01–1000.

## Custom palettes and alphabets

The `colors` string is a weighted list. Repeat a code to increase its weight.
Single letters remain supported even when the built-in profile uses two shades.

| Codes | Color pair |
| --- | --- |
| `gG` | Green |
| `bB` | Blue |
| `rR` | Red |
| `mM` | Purple |
| `yY` | Yellow |
| `oO` | Orange |
| `sS` | Grey |
| `c`, `w`, `k` | Cyan, white, black |

Lowercase is the brighter shade, uppercase the darker shade. Both use explicit
RGB from term-colors; cyan, white and black are local additions. Truecolor is the
current target; automatic reduced-depth adaptation is not implemented.

```toml
colors = "ggbB"
per_glyph_colors = true
highlight_probability = 0.12
glyph_set = "special"
```

Alphabets: `all`, `alpha`, `alpha_lower`, `alpha_upper`, `alphanum`, `binary`,
`decimal`, `hex`, `hex_lower`, `hex_upper`, `math`, `set`, `special`, `symbol`.
The default `special` contains halfwidth Katakana. Glyphs are single Unicode
scalars, not grapheme clusters; font support affects appearance.

## Command-line options

| Option | Environment equivalent |
| --- | --- |
| `--config` | `TERMATRIX_CONFIG` |
| `-c`, `--colors` | `TERMATRIX_COLORS` |
| `-g`, `--glyph-set` | `TERMATRIX_GLYPH_SET` |
| `-f`, `--max-refresh-rate` | `TERMATRIX_MAX_REFRESH_RATE` |
| `--seed` | None (default 42) |

`--help` and `--version` work without a terminal. The animation requires interactive
input/output and rejects `TERM=dumb`; it has no redirected-output mode.

## Upgrading from 2.x

Version 3 adds live reconfiguration, automatic persistence, resize handling,
independent overlapping traces, configurable highlights, and a density chart.
`-c`, `-g`, and `-f` remain; `--update-frequency` aliases `--max-refresh-rate`.
The refresh environment variable is now `TERMATRIX_MAX_REFRESH_RATE`.
`--min-step-delay` and `--max-step-delay` have been replaced by Min/Max speed in
the panel or `min_speed` / `max_speed` in TOML. They are not millisecond values.
Named-profile selection and management remain future work.
