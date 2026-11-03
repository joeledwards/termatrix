# Release checks for 3.2.0

Theme-aware gradients, automatic reduced-color adaptation with cached conversions,
tuned fresh defaults, and color previews within palette names. Saved preferences
remain supported. Palette management is deferred to the [follow-up plan](palette-manager-plan.md).

## Local checks

```sh
cargo test --locked --lib --bins
cargo fmt --check
cargo clippy --locked --lib --bins -- -D warnings
cargo build --locked
cargo package --locked --list
cargo package --locked
```

Workspace commands can use `../../scripts/with-rust`. term-colors 0.2.0 is
published. Both standalone source builds and packaged builds use the registry
version and its `Style::swatch()` API; no sibling repository is required.

Review package contents, license, changelog and README links. Keep local preview
settings, build artifacts and workspace tooling out of the package.

## Visual checks

Developer feedback accepted resize behavior, controls, the performance layout,
explicit tail shading and profile-picker colors. The agent has not run interactive terminal checks.
Before publishing, verify fresh defaults, saved preferences, pause/resume, all exit
keys, and terminal restoration on your target platforms. The backend supports
Linux/macOS/Windows; this is not a claim of platform-by-platform visual validation.

The README uses `docs/images/termatrix-3.png`, captured by the developer in iTerm
on macOS. Keep the repository URL in the README so the image displays on crates.io
after pushing. A screenshot records a particular configuration; panel details may
change in later builds.

See [coverage](coverage.md) for remaining test gaps. Profile rename/delete, graphical
frontends and custom palette management remain future work.

## Preparation evidence (3.2.0)

- All 62 isolated unit tests pass with offline locked dependencies.
- Formatting and Clippy with warnings denied pass; diff whitespace checks pass.
- Offline locked package verification passes (25 files), including a packaged build.
  `--allow-dirty` was used during preparation before committing.
- Developer feedback accepted gradients, light-theme effects, reduced-color cache
  responsiveness, and palette-name previews. The agent did not run interactive checks.
- Registry-connected dry run, publication, pushing and tagging have not been run.

To compare rendering manually (developer-run, not executed by the agent):

```sh
COLORTERM=truecolor termatrix --theme dark
TERM=xterm-256color COLORTERM= termatrix --theme dark
TERM=xterm COLORTERM= termatrix --theme dark
COLORTERM=truecolor termatrix --theme light
```

Check that the terminal background stays unchanged, tails remain faintly visible,
and ANSI/256-color animation stays responsive. Test a fresh profile using an
unused `--config` path, and verify existing preferences still load.

## Previous release preparation evidence (3.1.1)

- All 56 isolated unit tests pass offline with the locked dependencies.
- Formatting and Clippy with warnings denied pass.
- Offline locked package verification passes (24 files), including a packaged build.
  `--allow-dirty` was used because release preparation is not yet committed.
- Registry-connected publish dry run and publication have not been run; use the
  maintainer's private credential context after committing and pushing.

## Previous release preparation evidence (3.1.0)

- 56 isolated unit tests pass.
- Formatting, Clippy with warnings denied, and the build pass.
- Package verification compiled against published term-colors 0.2.0, with no local path override.
- Documentation links and the screenshot were checked locally. README image/document links
  use repository URLs so they remain usable on crates.io after the commit is pushed.
- The package includes README, license, changelog, user/design/release guides and the PNG screenshot.
- Registry-connected publish dry run and publication remain maintainer actions.

## Maintainer publication

Push reviewed commits, then use your private credential context:

```sh
cargo publish --locked --dry-run
cargo publish --locked
```

The agent does not push or publish. Keep publishing credentials out of the
workspace-local Cargo home. Create/push release tags according to your policy.
