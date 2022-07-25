["setup_local.exs", "utils.exs"] |> Enum.map(&Code.require_file/1)

defmodule Echoer do
  @moduledoc false

  use Ockam.Worker

  alias Ockam.Message
  alias Ockam.Router

  @impl true
  def handle_message(message, state) do
    IO.puts("\n[✓] Address: #{state.address}, Received: #{message.payload}")

    message
    |> Message.reply(state.address, Message.payload(message))
    |> Router.route()

    {:ok, state}
  end
end

require Logger

alias Ockam.Stream.Client.BiDirectional
alias Ockam.Vault

Logger.configure(level: :info)

# Initialize the TCP Transport.
Ockam.Transport.TCP.start()

# Create a Vault to safely store secret keys for Bob.
{:ok, vault} = Vault.Software.init()

# Create an Identity to represent Bob.
{:ok, bob} = Vault.secret_generate(vault, type: :curve25519)

# Create a secure channel listener for Bob that will wait for requests to
# initiate an Authenticated Key Exchange.
Ockam.SecureChannel.create_listener(
  vault: vault,
  identity_keypair: bob,
  address: "listener"
)

# Connect, over TCP, to the cloud node at `127.0.0.1:4000` and
# request the `stream_kinesis` service to create two Kinesis backed streams -
# `alice_to_bob` and `bob_to_alice`.
#
# After the streams are created, create:
# - a receiver (consumer) for the `alice_to_bob` stream
# - a sender (producer) for the `bob_to_alice` stream.
node_in_hub = Ockam.Transport.TCPAddress.new({127, 0, 0, 1}, 4000)
b_to_a_stream_address = Utils.unique_with_prefix("bob_to_alice")
a_to_b_stream_address = Utils.unique_with_prefix("alice_to_bob")
client_id = Utils.unique_with_prefix("bob")

{:ok, consumer} =
  BiDirectional.subscribe(
    a_to_b_stream_address,
    client_id,
    service_route: [node_in_hub, "stream_kinesis"],
    index_route: [node_in_hub, "stream_index"],
    partitions: 1
  )

Utils.wait(fn ->
  Ockam.Stream.Client.Consumer.ready?(consumer)
end)

BiDirectional.ensure_publisher(
  a_to_b_stream_address,
  b_to_a_stream_address,
  client_id,
  service_route: [node_in_hub, "stream_kinesis"],
  index_route: [node_in_hub, "stream_index"],
  partitions: 1
)

IO.puts("\n[✓] Streams were created on the node at: 127.0.0.1:4000")
IO.puts("\nbob_to_alice stream address is: #{b_to_a_stream_address}")
IO.puts("alice_to_bob stream address is: #{a_to_b_stream_address}\n")

{:ok, _echoer} = Echoer.create(address: "echoer")
