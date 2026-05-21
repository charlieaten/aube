defmodule Mix.Tasks.Aube do
  @moduledoc """
  Runs the Aube CLI.

  Every argument after `mix aube` is forwarded to Aube's native CLI bridge
  unchanged:

      mix aube install --frozen-lockfile
      mix aube run dev -- --watch

  The task does not shell out to an external executable.
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
