defmodule Mix.Tasks.Aube.Install do
  @moduledoc """
  Installs JavaScript packages with Aube.
  """

  use Mix.Task

  @shortdoc "Install JavaScript packages with Aube"

  @switches [
    cwd: :string,
    frozen_lockfile: :boolean,
    no_frozen_lockfile: :boolean,
    prefer_frozen_lockfile: :boolean,
    prod: :boolean,
    dev: :boolean,
    no_optional: :boolean,
    offline: :boolean,
    prefer_offline: :boolean,
    ignore_scripts: :boolean,
    lockfile_only: :boolean,
    force: :boolean,
    node_linker: :string,
    registry: :string
  ]

  @aliases [
    P: :prod,
    D: :dev
  ]

  @impl true
  def run(args) do
    case OptionParser.parse(args, strict: @switches, aliases: @aliases) do
      {opts, [], []} ->
        run_install(opts)

      {_opts, extra, invalid} ->
        details =
          (Enum.map(invalid, fn {flag, _} -> flag end) ++ extra)
          |> Enum.join(", ")

        Mix.raise("unknown or invalid aube.install option(s): #{details}")
    end
  end

  defp run_install(opts) do
    case Aube.install(opts) do
      {:ok, result} ->
        Mix.shell().info(
          "Installed JavaScript packages with aube in #{result.project_dir} (#{result.duration_ms} ms)"
        )

      {:error, error} ->
        suffix = if error.code, do: " [#{error.code}]", else: ""
        Mix.raise("aube install failed#{suffix}: #{error.message}")
    end
  end
end
