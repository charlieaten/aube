defmodule Mix.Tasks.AubeTest do
  use ExUnit.Case, async: false

  setup do
    on_exit(fn -> Mix.Task.reenable("aube") end)
  end

  test "runs aube through the native CLI bridge" do
    tmp = tmp_project!("aube_mix_task")

    Mix.Tasks.Aube.run([
      "--silent",
      "--dir",
      tmp,
      "install",
      "--lockfile-only",
      "--ignore-scripts"
    ])

    assert File.exists?(Path.join(tmp, "aube-lock.yaml"))
  end

  defp tmp_project!(prefix) do
    dir = Path.join(System.tmp_dir!(), "#{prefix}_#{System.unique_integer([:positive])}")
    File.mkdir_p!(dir)
    File.write!(Path.join(dir, "package.json"), ~s({"name":"#{prefix}","version":"1.0.0"}))
    dir
  end
end
