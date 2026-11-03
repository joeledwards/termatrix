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

Heads fade through the primary color toward black or white according to the
selected theme, with faintly visible tails; highlighted glyphs retain their
bright accents. See [trace fading](#trace-fading-and-terminal-capabilities). Hidden traces continue moving and can reappear. Startup is
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
profile names blue, creation actions green, and errors red. The profile picker and
palette manager share purple outlines and headings. The selected row has
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

Individual palette previews show one name in the primary color (for example,
**Green**), while mixed palettes show their colors within the name itself. Rainbow uses
red, orange, yellow, green, blue, and purple, repeating red for the final letter;
Christmas cycles red, green, and white. Cyan remains available in custom palettes.
Saved palette codes and shade choices remain supported.

| Codes | Palette |
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
RGB from term-colors; cyan, white and black are local additions. Rain adapts automatically to truecolor, 256-color, or ANSI terminals.

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
| `--theme auto\|dark\|light` | `TERMATRIX_THEME` |
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

## Trace fading and terminal capabilities

`--theme auto|dark|light` (environment: `TERMATRIX_THEME`) controls the rain
foreground fade; it does not set the terminal background. Auto uses the final ANSI background index in `COLORFGBG`: 7 and 9–15
select light; other or missing values select dark. No terminal background query
is sent. Explicit themes override detection. The controls retain their dark panel.

The head starts at white (dark theme) or black (light theme), reaches the chosen
trace/glyph color at 30% of the trace, and fades toward black (dark theme) or white (light theme) at its tail. The tail retains 15% of the primary color with a small contrast
floor, so the final glyph stays visible. Reduced-color approximation excludes
the black/white fade endpoint for trailing glyphs.
A one-glyph trace is a head. Clipping does not restart the gradient. Highlighted
glyphs keep their existing probability and lifetime, with a strong lift toward
the head color. Position-based highlights are not currently provided.

`COLORTERM=truecolor|24bit` (case-insensitive) or a `TERM` containing `direct`
selects RGB. Otherwise `TERM` containing `256color` selects the nearest xterm
256-color entry; other capable terminals use the nearest standard ANSI entry.
Terminal-customized ANSI palettes can affect these approximations.

## Fresh-profile defaults

Speed ranges from 2 to 12 with uniform bias (0), using canvas-relative speed.
Density is 2.0 and clumping is 0.8. Traces contain 4–40 glyphs, mutation rate
is 0.05, and the refresh cap is 60 Hz. The palette is Green (`gG`), alphabet
is Special (Katakana), individual glyph colors are off, and highlights occur
with 2% probability. Chart compression is 6 and the controls panel starts visible.
Saved preferences override defaults; omitted settings use these defaults.

## Palette manager

Press `c` to manage the shared palette collection. The manager shows `[x]` for
visible palettes and `[ ]` for hidden ones. Use ↑/↓ to navigate and preview the
selected palette on the rain, Enter to apply it, or Esc (or `c`) to return without
changing your active palette. The animation keeps running; a paused scene also
updates its preview. Ctrl+C still quits.

- Space toggles visibility in the Palette setting's left/right cycle. Hidden
  palettes remain selectable in the manager and load normally from profiles.
  At least one palette must remain visible.
- `n` creates a palette using the active colors as its starting point. Type a name,
  press Tab to edit color codes, and press Enter to save. Tab or ↑/↓ switches fields;
  Backspace edits and Esc cancels. Names must be unique ignoring case, have 1–64
  printable characters, and have no surrounding spaces.
- Codes are `bBcgGkmMrRwyYoOsS`, as in existing `--colors` and profile settings.
  Use 1–256 codes; repetitions provide weight (for example `rrrg` gives red
  three quarters of the choices). Invalid colors produce a save error.
- `e` edits a custom palette. `d` requests deletion; `y` confirms and any other
  key cancels. Built-ins cannot be edited or deleted.

The catalog is `termatrix/palettes.toml` in the platform configuration directory
(the same root that normally contains `profiles/`). It is shared even when
`--config` selects a profile elsewhere. Missing catalogs use the built-ins with
all entries visible; merely opening the manager does not create a file.
Changes save immediately using atomic replacement. Errors keep the previous
catalog and show a message. Profile autosave and catalog saves are independent.

Profiles and CLI overrides continue storing raw color codes, not catalog names.
Editing or deleting a custom palette does not change already saved or active
profile colors; select the updated palette and let the profile save to adopt it.
Equivalent code sequences are allowed under different names; the panel and
manager initially identify the first matching catalog entry. Unknown sequences
remain usable as custom colors. Hiding never rewrites a profile.

Example catalog (built-ins are implicit):

```toml
hidden = ["Christmas", "Grey"]

[[custom]]
name = "Sunset"
colors = "rrroY"
```

Arbitrary RGB hex colors are deferred.

The controls-panel command hints wrap to fit the available width. Extra hint
rows reduce the visible settings list; scrolling keeps the selected setting in
view. Very small panels can still lack room for all help and status rows.
