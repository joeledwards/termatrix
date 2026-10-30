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
| Refresh cap | Maximum display updates per second (1–240 Hz), shown with units; independent of animation speed |
| Canvas-relative | On scales advancement with canvas height; Off uses glyph-relative speed |
| Palette | Built-in colors or the custom sequence loaded from TOML |
| Alphabet | Katakana, letters, numbers, symbols, and other character sets |
| Individual glyph colors | Off (default) chooses colors per trace; On chooses per glyph |
| Highlight probability | Chance a glyph receives a stable bright/bold accent; 0 disables accents |
| Chart compression | 0 is linear; higher values expand recent history (0–10, default 6; arrows adjust by 0.5) |

Heads remain white. The older half of each trace uses 60% of its selected RGB
channel values for consistent tail shading across terminals; highlighted glyphs
retain their bright accents. Hidden traces continue moving and can reappear. Startup is
empty; coverage grows as traces enter from the top. The chart measures visible
coverage after occlusion, not the population target controlled by Density.
It samples at 5 Hz, retains ten minutes, and counts overlapping glyphs once.
The time axis runs from ten minutes ago on the left to now on the right.
Compression changes the logarithmic time scale immediately without clearing history;
axis labels show actual ages. Set `chart_compression = 6.0` in TOML or adjust
Chart compression in the panel. Older files without this setting use 6.0.
History continues while paused or hidden. Resizing changes the measured area.

## Performance panel

Controls use 72 columns when the terminal is at least 96 columns wide; performance
uses all remaining width. Narrower terminals place a compact summary below the
controls. Very short terminals omit performance to preserve usable controls.

The summary shows current active traces, actual rendered frames per second, and
average frame work in milliseconds. Work is elapsed time spent in the active loop
(simulation, projection, telemetry, saves, and rendering/output), accumulated per
completed frame; deliberate event/refresh waiting is excluded. It is a wall-time
measurement, not CPU utilization. Pausing freezes simulation while FPS continues.
Values aggregate over intervals of at least 200 ms; low refresh caps can produce
zero-frame intervals. Metrics continue collecting when the controls are hidden.

With at least seven plot rows available, performance widths of 40, 80, and 120
columns show one, two, or three plots: frame work, glyph coverage, then active traces.
Additional width widens the plots. All plots share the ten-minute history and
compression setting. Coverage stays at 0–100%; trace and work scales fit the
retained history with headroom. The summary remains available without plots.

## Saved preferences

Default locations use the platform configuration directory:

- Linux: `$XDG_CONFIG_HOME/termatrix/profiles/default.toml`, usually under `~/.config`.
- macOS: `~/Library/Application Support/termatrix/profiles/default.toml`.
- Windows: `%APPDATA%\termatrix\config\profiles\default.toml`.

`--config <path>` or `TERMATRIX_CONFIG` overrides the location. The panel shows
`Config Profile: <name>`, using the filename without its final extension (for
example, `default.toml` displays `default`). Saved, pending, and failure feedback
appears beside the name. Changes save after
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

## Selecting and creating profiles

Press `p` anywhere in the animation to open the profile picker, including when
controls are hidden. Use arrows to select and Enter to load. Press `n`, or select
**Create new profile**, to copy the current settings to a new named profile.
Type a name without `.toml` and press Enter; Backspace edits, Esc returns to the
list, and Esc or `p` in the list closes it. Keys are yellow, descriptions white,
profile names blue, creation actions green, and errors red. The selected row has
a yellow pointer and a shaded background. Ctrl+C still exits the application.
Other animation shortcuts are captured while the picker is open. Animation continues.

Profiles are `.toml` files beside the current configuration file. This is normally
`termatrix/profiles/`; `--config` uses that file's directory instead. Names may
contain 1–64 letters, digits, spaces, hyphens and underscores, without leading or
trailing spaces; reserved device names are rejected. Existing files are never
overwritten by Create. The list scrolls and refreshes each time it is opened.

Switching validates the destination and saves pending changes to the old profile
before activating the new one. Save/load errors stay in the picker and retain the
active profile. Subsequent edits save to the newly selected file. Switching keeps
the running scene and applies settings live; it does not reset the animation.
The next launch still opens `default.toml` unless you specify `--config`; selection
is not remembered across launches. Rename/delete files outside the application.

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
Use `p` to switch or create profiles; renaming and deleting remain manual file operations.
