# Release checks for 3.0.0

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

Developer feedback accepted resize behavior, controls, the coverage chart and
palette/highlight appearance. The agent has not run interactive terminal checks.
Before publishing, verify fresh defaults, saved preferences, pause/resume, all exit
keys, and terminal restoration on your target platforms. The backend supports
Linux/macOS/Windows; this is not a claim of platform-by-platform visual validation.

The README uses `docs/images/termatrix-3.png`, captured by the developer in iTerm
on macOS. Keep the repository URL in the README so the image displays on crates.io
after pushing. A screenshot records a particular configuration; panel details may
change in later builds.

See [coverage](coverage.md) for remaining test gaps. Named profiles, graphical
frontends and automatic reduced-color adaptation remain future work.

## Preparation evidence

- 45 isolated unit tests pass.
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
