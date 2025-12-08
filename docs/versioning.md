# Versioning & Releases

I'm keeping things simple and SemVer-ish while we're pre-1.0.

## Strategy
- Use `0.x.y` while things are moving fast.
- Bump:
  - `x` for breaking changes (0.4 → 0.5),
  - `y` for features/fixes (0.4.1 → 0.4.2).
- Tag releases as `v0.x.y` (include the `v`).

## How to cut a release
1) Make sure main is green:
```bash
cargo fmt --all
cargo check -p boomai-daemon
cargo test --workspace   # if feasible
```
2) Tag it:
```bash
git tag v0.1.0
git push origin v0.1.0
```
3) GitHub Actions will:
   - run fmt/check/test on the tag
   - create a GitHub release for that tag (no binaries yet)

## What’s shipped
- Source-only releases for now (no prebuilt binaries).
- When we’re ready, we can add Tauri/Rust binaries to the release job.

## Changelog
- Keep `CHANGELOG.md` (future) or release notes updated when tagging.


