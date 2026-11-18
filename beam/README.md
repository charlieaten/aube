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
Check that the GitHub release tag matches the version in `beam/mix.exs` and
that the workflow produced a checksum for every platform in the build matrix.
Use the generated `beam/checksum-Elixir.Aube.Native.exs` when staging the Hex
package.

From the repository root, stage the package and validate it locally:

```sh
beam/scripts/stage-hex-package.bash
cd target/beam-hex-package
mix hex.build
```
