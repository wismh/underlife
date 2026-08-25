# underlife (`pseudo3d`)

OpenGL 3.3 GPU raycaster (Wolfenstein-style DDA). Crate: `pseudo3d`.

- Architecture: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- Work plan: [docs/PLAN.md](docs/PLAN.md)

Build: Rust **≥ 1.88** (lockfile MSRV; `edition2024` deps appeared at 1.85), then `git lfs pull` and `cargo build --release`.

PNG, OGG, and MP3 files are stored in Git LFS. Without LFS smudge (for example a clone with `GIT_LFS_SKIP_SMUDGE`, or skipping `git lfs pull`) those assets stay as pointer files, and image/audio loaders will fail. Run `git lfs pull` after clone.
