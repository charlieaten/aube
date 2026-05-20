defmodule Aube.MixProject do
  use Mix.Project

  @version "0.1.0"
  @source_url "https://github.com/endevco/aube"

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
      files: [
        "lib",
        "native/aube_nif/.cargo",
        "native/aube_nif/src",
        "native/aube_nif/Cargo.toml",
        "crates",
        "Cargo.toml",
        "Cargo.lock",
        "checksum-*.exs",
        "README.md",
        "LICENSE",
        "mix.exs"
      ]
    ]
  end

  defp docs do
    [
      main: "Aube",
      source_url: @source_url,
      extras: ["README.md"]
    ]
  end
end
