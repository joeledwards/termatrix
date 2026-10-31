# Release checks for 3.1.1

This documentation-only patch refreshes the screenshots and adds the README
Profiles section. Runtime behavior is unchanged from 3.1.0.

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
frontends and automatic reduced-color adaptation remain future work.

## Preparation evidence (3.1.1)

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
