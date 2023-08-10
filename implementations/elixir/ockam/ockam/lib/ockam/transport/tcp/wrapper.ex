defmodule Ockam.Transport.TCP.Wrapper do
  def send(client, socket, data) do
    client.send(socket, data)
  end
end
