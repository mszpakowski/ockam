["setup_local.exs", "utils.exs"] |> Enum.map(&Code.require_file/1)

require Logger

alias Ockam.Stream.Client.BiDirectional
alias Ockam.Vault

Logger.configure(level: :info)

# Register this process as worker address "app".
Ockam.Node.register_address("app")

# Initialize the TCP Transport.
Ockam.Transport.TCP.start()

# Create a Vault to safely store secret keys for Bob.
{:ok, vault} = Vault.Software.init()

# Create an Identity to represent Bob.
{:ok, alice} = Vault.secret_generate(vault, type: :curve25519)

# This program expects that Bob has created two streams
# bob_to_alice and alice_to_bob on the cloud node at 127.0.0.1:4000
# We need the user to provide the addresses of these streams.

# From standard input, read bob_to_alice stream address.
IO.puts("\nEnter the bob_to_alice stream address: ")
b_to_a_stream_address = IO.read(:stdio, :line)
b_to_a_stream_address = String.trim(b_to_a_stream_address)

# From standard input, read alice_to_bob stream address.
IO.puts("\nEnter the alice_to_bob stream address: ")
a_to_b_stream_address = IO.read(:stdio, :line)
a_to_b_stream_address = String.trim(a_to_b_stream_address)

# We now know that the route to:
# - send messages to bob is [(TCP, "127.0.0.1:4000"), a_to_b_stream_address]
# - receive messages from bob is [(TCP, "127.0.0.1:4000"), b_to_a_stream_address]

# Starts a sender (producer) for the alice_to_bob stream and a receiver (consumer)
# for the `bob_to_alice` stream to get two-way communication.
node_in_hub = Ockam.Transport.TCPAddress.new({127, 0, 0, 1}, 4000)
client_id = Utils.unique_with_prefix("alice")

{:ok, consumer} =
  BiDirectional.subscribe(
    b_to_a_stream_address,
    client_id,
    service_route: [node_in_hub, "stream_kinesis"],
    index_route: [node_in_hub, "stream_index"],
    partitions: 1
  )

Utils.wait(fn ->
  Ockam.Stream.Client.Consumer.ready?(consumer)
end)

{:ok, sender} =
  BiDirectional.ensure_publisher(
    b_to_a_stream_address,
    a_to_b_stream_address,
    client_id,
    service_route: [node_in_hub, "stream_kinesis"],
    index_route: [node_in_hub, "stream_index"],
    partitions: 1
  )

route = [sender, "listener"]
{:ok, channel} = Ockam.SecureChannel.create(route: route, vault: vault, identity_keypair: alice)

IO.puts("\n[✓] End-to-end encrypted secure channel was established.\n")

defmodule Loop do
  alias Ockam.Message
  alias Ockam.Router

  def run(channel) do
    # Read a message from standard input.
    IO.puts("Type a message for Bob's echoer:")
    message = IO.read(:stdio, :line)
    message = String.trim(message)

    # Send the provided message, through the channel, to Bob's echoer.
    msg = %{
      onward_route: [channel, "echoer"],
      return_route: ["app"],
      payload: message
    }

    Router.route(msg)

    # Wait to receive an echo and print it.
    receive do
      reply ->
        IO.puts("\nAlice received an echo: #{reply.payload}\n")
        run(channel)
    end
  end
end

Loop.run(channel)
