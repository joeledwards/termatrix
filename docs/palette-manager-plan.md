# Palette manager: 3.3.0 implementation

Implemented for 3.3.0 after the 3.2.0 gradient release. See the
[user guide](guide.md#palette-manager) for the implemented contract.

## Agreed scope

- A palette manager alongside the profile picker, reusing its navigation and
  modal interaction patterns; `c` opens the palette manager.
- Create, edit and delete named custom palettes built from existing color codes.
  Preserve weighted codes and preview the name in its colors plus the live fade.
- Show/hide built-in and custom palettes to control the Palette setting's cycle.
  Keep at least one palette visible. Hiding affects cycling, while existing
  profiles can still load and render a hidden palette.
- Persist a shared palette catalog and visibility preferences across profiles.
  Each profile retains its selected palette. Preserve existing color-code files
  and CLI overrides; the catalog uses custom name/colors entries plus hidden names.
- Keep arbitrary RGB hex editing out of the first iteration. That requires
  extending the current single-character color representation.

## Implementation and validation

The catalog replaces the fixed built-in cycling list. Validated atomic saves
preserve the previous catalog on failure, and the active scene stays stable
while editing. Profiles retain raw codes: editing/deleting a catalog entry does
not change their saved or active appearance. Save and validation failures appear
in the manager.

Isolated tests cover catalog defaults/migration, weighted color validation, name collisions,
create/edit/delete, visibility persistence, last-visible protection, hidden
selection loading, profile compatibility, keyboard navigation, styled previews,
and save failure preservation. The behavior coverage matrix and guide describe
the shipped contract. Real-terminal checks remain developer-run.
