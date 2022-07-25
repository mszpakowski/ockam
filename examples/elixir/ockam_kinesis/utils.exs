defmodule Utils do
  @moduledoc false

  def wait(fun) do
    case fun.() do
      true ->
        :ok

      false ->
        :timer.sleep(100)
        wait(fun)
    end
  end

  def unique_with_prefix(prefix) do
    suffix = 4 |> :crypto.strong_rand_bytes() |> Base.encode16(case: :lower, padding: false)

    "#{prefix}_#{suffix}"
  end
end
