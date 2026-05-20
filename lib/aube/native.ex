defmodule Aube.Native do
  @moduledoc false

  @version Mix.Project.config()[:version]
  @force_build System.get_env("AUBE_BUILD") in ["1", "true", "TRUE", "yes", "YES"]
  @cache_opts (if @force_build do
                 []
               else
                 checksum_path = Path.expand("../../checksum-Elixir.Aube.Native.exs", __DIR__)

                 cache_salt_source =
                   if File.exists?(checksum_path), do: File.read!(checksum_path), else: @version

                 cache_salt =
                   :crypto.hash(:sha256, cache_salt_source)
                   |> Base.encode16(case: :lower)
                   |> binary_part(0, 16)

                 base_cache_dir =
                   Path.join(
                     :filename.basedir(:user_cache, "aube/rustler_precompiled"),
                     cache_salt
                   )

                 [base_cache_dir: base_cache_dir]
               end)
  @targets [
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-musl",
    "x86_64-unknown-linux-musl",
    "x86_64-pc-windows-msvc"
  ]
  @rustler_precompiled_opts Keyword.merge(
                              [
                                otp_app: :aube,
                                crate: "aube_nif",
                                base_url:
                                  "https://github.com/charlieaten/aube/releases/download/main",
                                force_build: @force_build,
                                mode: if(Mix.env() == :prod, do: :release, else: :debug),
                                env: [{"AUBE_SKIP_PRIMER_GENERATION", "1"}],
                                version: @version,
                                nif_versions: ["2.15"],
                                targets: @targets
                              ],
                              @cache_opts
                            )

  use RustlerPrecompiled, @rustler_precompiled_opts

  def install(_opts), do: :erlang.nif_error(:nif_not_loaded)
end
