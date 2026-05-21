# Aube for Elixir

Elixir bindings for the Aube JavaScript package manager.

This package loads Aube through a Rustler NIF and exposes a small Elixir API:

```elixir
Aube.install(cwd: "assets")
Aube.run(["install", "--frozen-lockfile"])
```

It also exposes the CLI through Mix without shelling out:

```sh
mix aube install --frozen-lockfile
mix aube run dev -- --watch
```

The NIF bridge lives in the Rust workspace at `crates/aube-beam-nif`; this
directory contains the BEAM/Hex distribution surface.
