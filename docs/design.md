# Animation and rendering design

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
- White bold heads and darker tails are fixed appearance rules. Tail RGB channels
  are scaled to 60% (rounded), without the terminal-dependent dim attribute. Selected body/tail
  glyphs are highlighted with bold and an explicitly brighter RGB shade.
  `highlight_probability` (0–1, default 0.12) adjusts this live; each glyph
  retains its random highlight percentile through mutation and palette changes. The
  weighted palette uses `bBcgGkmMrRwyYoOsS` (`o` is orange). Repeated letters
  increase their weight. Color choices are stable per trace, or per glyph with
  `per_glyph_colors = true` (the Individual glyph colors control). Head highlighting
  remains white in both modes. The compact panel list includes Green, Blue, Red, Purple, Yellow, Orange, Rainbow,
  Purple-green-blue, Orange-green-blue, Christmas (red/green/white), and
  Grey. TOML accepts custom sequences. All named colors use explicit RGB. The shared `term-colors` dark palette
  provides red, orange, yellow, green, blue, purple and grey; local additions
  are cyan (51,187,204), white (238,238,238), and black (34,34,34).
- Terminal lanes allocate two cells for a Unicode scalar of width one or two.
  Grapheme clusters are not yet supported. Shrinking the canvas removes traces
  outside its columns or below its bottom. Showing the panel reduces canvas height.


The simulation has no terminal types in its API. The CLI owns cell layout,
projection, history sampling, terminal output and preference persistence.
See [test coverage](coverage.md) for current limitations.
