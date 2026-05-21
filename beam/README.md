# Aube for Elixir

Elixir bindings for the Aube JavaScript package manager.

This package loads Aube's stable embedded installer through a Rustler NIF. It
does not shell out to an `aube` executable:

```elixir
Aube.install(cwd: "assets")
Aube.install(cwd: ".", frozen_lockfile: true, ignore_scripts: true)
```

The NIF bridge lives in the Rust workspace at `crates/aube-beam-nif`; this
directory contains the BEAM/Hex distribution surface.
