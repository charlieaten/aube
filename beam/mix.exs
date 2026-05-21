defmodule Aube.MixProject do
  use Mix.Project

  @version "1.15.0-beta.3"
  @source_url "https://github.com/charlieaten/aube"

  def project do
    [
      app: :aube,
      version: @version,
      elixir: "~> 1.15",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      description: "Elixir bindings for the Aube JavaScript package manager.",
      package: package(),
      docs: docs()
    ]
  end

  def application do
    [
      extra_applications: [:logger]
    ]
  end

  defp deps do
    [
      {:rustler, "~> 0.37", optional: true},
      {:rustler_precompiled, "~> 0.9"}
    ]
  end

  defp package do
    [
      licenses: ["MIT"],
      links: %{"GitHub" => @source_url},
      files:
        [
          ".formatter.exs",
          "lib",
          "README.md",
          "LICENSE",
          "mix.exs"
        ] ++ checksum_files() ++ source_build_files()
    ]
  end

  defp checksum_files do
    case Path.wildcard("checksum-*.exs") do
      [] -> []
      _files -> ["checksum-*.exs"]
    end
  end

  defp source_build_files do
    if File.dir?("crates") do
      ["Cargo.toml", "Cargo.lock", "crates"]
    else
      []
    end
  end

  defp docs do
    [
      main: "Aube",
      source_url: @source_url,
      extras: ["README.md"]
    ]
  end
end
