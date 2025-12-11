# Versioning & Releases

I'm keeping things simple and SemVer-ish while we're pre-1.0.

## Strategy
- Use `0.x.y` while things are moving fast.
- Bump:
  - `x` for breaking changes (0.4 → 0.5),
  - `y` for features/fixes (0.4.1 → 0.4.2).
- Tag releases as `v0.x.y` (include the `v`).

## How to cut a release (source-only for now)
1) Make sure main is green (matches CI gates):
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run -p boomai-daemon --all-targets   # or cargo test --workspace
cargo audit
cargo deny check
```
2) Tag it (example for v0.2.5):
```bash
git tag v0.2.5
git push origin v0.2.5
```
3) GitHub Actions will:
   - run fmt/clippy/test on the tag (see `ci.yml`)
   - create a GitHub release for that tag (no binaries yet; desktop/tauri packaging is future work)

## What’s shipped
- Source-only releases for now (no prebuilt binaries).
- When we’re ready, we can add Tauri/Rust binaries + artifact attestations to the release job.

## Changelog
- Keep `CHANGELOG.md` (future) or release notes updated when tagging.



