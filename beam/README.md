# Aube for Elixir

Elixir bindings for the Aube JavaScript package manager.

This package loads Aube's stable embedded installer through a Rustler NIF. It
does not shell out to an `aube` executable:

```elixir
Aube.install(cwd: "assets")
Aube.install(cwd: ".", frozen_lockfile: true, ignore_scripts: true)
```

It also exposes aube's complete one-shot CLI through Mix:

```sh
mix aube install
mix aube update
mix aube run dev -- --watch
```

The NIF bridge lives in the Rust workspace at `crates/aube-beam-nif`; this
directory contains the BEAM/Hex distribution surface.

## Preparing a Hex release

Run the `beam-nif-release` GitHub Actions workflow for the release commit.
A branch build outside `main` produces the platform archives and the
`rustler-precompiled-checksum` artifact. To place those archives at the
URL used by Rustler, run the workflow for the same commit with
`publish_assets` enabled, or push the release commit to `main`.

Check that the GitHub release tag matches the version in `beam/mix.exs`
and that the checksum artifact covers every platform in the build matrix.
Copy the matching `checksum-Elixir.Aube.Native.exs` into `beam/` before
staging the Hex package.

From the repository root, stage the package and validate it locally:

```sh
beam/scripts/stage-hex-package.bash
cd target/beam-hex-package
mix hex.build
```
