defmodule Aube do
  @moduledoc """
  Elixir bindings for the Aube JavaScript package manager.
  """

  @type install_option ::
          {:cwd, Path.t()}
          | {:frozen_lockfile, boolean()}
          | {:no_frozen_lockfile, boolean()}
          | {:prefer_frozen_lockfile, boolean()}
          | {:prod, boolean()}
          | {:dev, boolean()}
          | {:no_optional, boolean()}
          | {:offline, boolean()}
          | {:prefer_offline, boolean()}
          | {:ignore_scripts, boolean()}
          | {:lockfile_only, boolean()}
          | {:force, boolean()}
          | {:node_linker, String.t()}
          | {:registry, String.t()}

  @type install_result :: %{
          project_dir: String.t(),
          duration_ms: non_neg_integer()
        }

  @type install_error :: %{
          message: String.t(),
          code: String.t() | nil
        }

  @type run_result :: %{
          exit_code: integer()
        }

  @type run_error :: %{
          message: String.t(),
          code: String.t() | nil,
          exit_code: integer()
        }

  @allowed_options [
    :cwd,
    :frozen_lockfile,
    :no_frozen_lockfile,
    :prefer_frozen_lockfile,
    :prod,
    :dev,
    :no_optional,
    :offline,
    :prefer_offline,
    :ignore_scripts,
    :lockfile_only,
    :force,
    :node_linker,
    :registry
  ]

  @doc """
  Installs JavaScript packages for a project with Aube.
  """
  @spec install([install_option()]) :: {:ok, install_result()} | {:error, install_error()}
  def install(opts \\ [])

  def install(opts) when is_list(opts) do
    with :ok <- validate_known_options(opts),
         :ok <- validate_mutual_exclusion(opts),
         {:ok, cwd} <- resolve_cwd(opts) do
      Aube.Native.install(native_options(opts, cwd))
    end
  end

  def install(_opts) do
    {:error, %{message: "install options must be a keyword list", code: "invalid_options"}}
  end

  @doc """
  Runs the Aube CLI with argv-style arguments.
  """
  @spec run([String.t()]) :: {:ok, run_result()} | {:error, run_error()}
  def run(args) when is_list(args) do
    if Enum.all?(args, &is_binary/1) do
      Aube.Native.run_cli(args)
    else
      {:error,
       %{
         message: "run args must be a list of strings",
         code: "invalid_args",
         exit_code: 1
       }}
    end
  end

  def run(_args) do
    {:error,
     %{
       message: "run args must be a list of strings",
       code: "invalid_args",
       exit_code: 1
     }}
  end

  defp validate_known_options(opts) do
    case Keyword.keys(opts) -- @allowed_options do
      [] ->
        :ok

      unknown ->
        {:error,
         %{
           message: "unknown install option(s): #{Enum.map_join(unknown, ", ", &inspect/1)}",
           code: "invalid_options"
         }}
    end
  end

  defp validate_mutual_exclusion(opts) do
    frozen_count =
      Enum.count(
        [:frozen_lockfile, :no_frozen_lockfile, :prefer_frozen_lockfile],
        &truthy?(opts[&1])
      )

    cond do
      frozen_count > 1 ->
        {:error,
         %{message: "frozen lockfile options are mutually exclusive", code: "invalid_options"}}

      truthy?(opts[:prod]) and truthy?(opts[:dev]) ->
        {:error,
         %{message: "prod and dev install modes are mutually exclusive", code: "invalid_options"}}

      truthy?(opts[:offline]) and truthy?(opts[:prefer_offline]) ->
        {:error,
         %{message: "offline and prefer_offline are mutually exclusive", code: "invalid_options"}}

      true ->
        :ok
    end
  end

  defp resolve_cwd(opts) do
    case Keyword.fetch(opts, :cwd) do
      {:ok, cwd} ->
        cwd
        |> Path.expand()
        |> validate_cwd()

      :error ->
        cond do
          File.exists?("assets/package.json") ->
            {:ok, Path.expand("assets")}

          File.exists?("package.json") ->
            {:ok, File.cwd!()}

          true ->
            {:error,
             %{
               message:
                 "no package.json found; pass :cwd or run from a project with package.json",
               code: "missing_package_json"
             }}
        end
    end
  end

  defp validate_cwd(cwd) do
    if File.dir?(cwd) do
      {:ok, cwd}
    else
      {:error, %{message: "cwd is not a directory: #{cwd}", code: "invalid_cwd"}}
    end
  end

  defp native_options(opts, cwd) do
    %{
      cwd: cwd,
      frozen_lockfile: truthy?(opts[:frozen_lockfile]),
      no_frozen_lockfile: truthy?(opts[:no_frozen_lockfile]),
      prefer_frozen_lockfile: truthy?(opts[:prefer_frozen_lockfile]),
      prod: truthy?(opts[:prod]),
      dev: truthy?(opts[:dev]),
      no_optional: truthy?(opts[:no_optional]),
      offline: truthy?(opts[:offline]),
      prefer_offline: truthy?(opts[:prefer_offline]),
      ignore_scripts: truthy?(opts[:ignore_scripts]),
      lockfile_only: truthy?(opts[:lockfile_only]),
      force: truthy?(opts[:force]),
      node_linker: opts[:node_linker],
      registry: opts[:registry]
    }
  end

  defp truthy?(value), do: value in [true, "true", "1", 1]
end
