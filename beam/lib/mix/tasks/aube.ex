defmodule Mix.Tasks.Aube do
  @moduledoc """
  Runs the Aube CLI inside the BEAM VM.

  Every argument after `mix aube` is forwarded unchanged:

      mix aube install --frozen-lockfile
      mix aube update
      mix aube run dev -- --watch
  """

  use Mix.Task

  @shortdoc "Run the Aube CLI"

  @impl true
  def run(args) do
    case Aube.run(args) do
      {:ok, %{exit_code: 0}} ->
        :ok

      {:ok, %{exit_code: exit_code}} ->
        System.halt(exit_code)

      {:error, error} ->
        suffix = if error.code, do: " [#{error.code}]", else: ""
        IO.puts(:stderr, "aube failed#{suffix}: #{error.message}")
        System.halt(error.exit_code || 1)
    end
  end
end
