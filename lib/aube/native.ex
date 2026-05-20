defmodule Aube.Native do
  @moduledoc false

  @version Mix.Project.config()[:version]
  @force_build System.get_env("AUBE_BUILD") in ["1", "true", "TRUE", "yes", "YES"]

  use RustlerPrecompiled,
    otp_app: :aube,
    crate: "aube_nif",
    base_url: "https://github.com/endevco/aube/releases/download/elixir-v#{@version}",
    force_build: @force_build,
    mode: if(Mix.env() == :prod, do: :release, else: :debug),
    env: [{"AUBE_SKIP_PRIMER_GENERATION", "1"}],
    version: @version,
    nif_versions: ["2.15"],
    targets: [
      "aarch64-apple-darwin",
      "x86_64-apple-darwin",
      "aarch64-unknown-linux-gnu",
      "x86_64-unknown-linux-gnu",
      "aarch64-unknown-linux-musl",
      "x86_64-unknown-linux-musl",
      "x86_64-pc-windows-msvc"
    ]

  def install(_opts), do: :erlang.nif_error(:nif_not_loaded)
end
