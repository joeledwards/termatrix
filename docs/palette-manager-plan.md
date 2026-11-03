# Palette manager: follow-up release

Implement after the 3.2.0 gradient release, as a separate feature and release.

## Agreed scope

- A palette manager alongside the profile picker, reusing its navigation and
  modal interaction patterns. Choose a discoverable shortcut during implementation.
- Create, edit and delete named custom palettes built from existing color codes.
  Preserve weighted codes and preview the name in its colors plus the live fade.
- Show/hide built-in and custom palettes to control the Palette setting's cycle.
  Keep at least one palette visible. Hiding affects cycling, while existing
  profiles can still load and render a hidden palette.
- Persist a shared palette catalog and visibility preferences across profiles.
  Each profile retains its selected palette. Preserve existing color-code files
  and CLI overrides; settle the on-disk schema before implementation.
- Keep arbitrary RGB hex editing out of the first iteration. That requires
  extending the current single-character color representation.

## Implementation and validation

Replace the fixed built-in cycling list with catalog lookup. Use validated,
atomic persistence and keep the active scene stable while editing. Decide how
editing/deleting a palette affects profiles that already reference it; existing
profiles must remain usable. Surface save and validation failures in the manager.

Cover catalog defaults/migration, weighted color validation, name collisions,
create/edit/delete, visibility persistence, last-visible protection, hidden
selection loading, profile compatibility, keyboard navigation, styled previews,
and save failure preservation with isolated tests. Update the behavior coverage
matrix and guide. Real-terminal checks remain developer-run.
