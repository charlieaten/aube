defmodule Mix.Tasks.Aube.InstallTest do
  use ExUnit.Case, async: false

  setup context do
    previous_shell = Mix.shell()

    if context[:capture_mix_shell] do
      Mix.shell(Mix.Shell.Process)
    end

    on_exit(fn ->
      Mix.shell(previous_shell)
      Mix.Task.reenable("aube.install")
    end)
  end

  test "unknown flags are rejected" do
    assert_raise Mix.Error, ~r/unknown or invalid aube.install option/, fn ->
      Mix.Tasks.Aube.Install.run(["--unknown"])
    end
  end

  @tag :capture_mix_shell
  test "runs install for explicit cwd" do
    tmp = tmp_project!("aube_mix_task")

    Mix.Tasks.Aube.Install.run(["--cwd", tmp])

    assert_received {:mix_shell, :info, [message]}
    assert message =~ "Installed JavaScript packages with aube"
    assert message =~ canonical_path(tmp)
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
