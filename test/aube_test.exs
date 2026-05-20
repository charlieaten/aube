defmodule AubeTest do
  use ExUnit.Case, async: false

  test "invalid cwd returns a clean error" do
    cwd = Path.join(System.tmp_dir!(), "aube-missing-#{System.unique_integer([:positive])}")

    assert {:error, %{code: "invalid_cwd", message: message}} = Aube.install(cwd: cwd)
    assert message =~ "cwd is not a directory"
  end

  test "unknown install options are rejected" do
    assert {:error, %{code: "invalid_options", message: message}} =
             Aube.install(cwd: ".", mystery: true)

    assert message =~ ":mystery"
  end

  test "mutually exclusive options are rejected before native install" do
    assert {:error, %{code: "invalid_options", message: message}} =
             Aube.install(cwd: ".", prod: true, dev: true)

    assert message =~ "mutually exclusive"
  end

  test "install succeeds for an empty JavaScript project" do
    tmp = tmp_project!("aube_elixir_success")

    assert {:ok, result} = Aube.install(cwd: tmp)
    assert result.project_dir == canonical_path(tmp)
    assert is_integer(result.duration_ms)
  end

  test "repeated installs in one BEAM VM are stable" do
    tmp = tmp_project!("aube_elixir_repeat")

    assert {:ok, _} = Aube.install(cwd: tmp)
    assert {:ok, _} = Aube.install(cwd: tmp)
  end

  defp tmp_project!(prefix) do
    dir = Path.join(System.tmp_dir!(), "#{prefix}_#{System.unique_integer([:positive])}")
    File.mkdir_p!(dir)
    File.write!(Path.join(dir, "package.json"), ~s({"name":"#{prefix}","version":"1.0.0"}))
    dir
  end

  defp canonical_path(path) do
    case Path.expand(path) do
      "/var/" <> rest -> "/private/var/" <> rest
      path -> path
    end
  end
end
