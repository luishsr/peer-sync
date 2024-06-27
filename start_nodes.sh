#!/bin/bash

# Usage: ./start_nodes.sh <number_of_nodes>

# Check if the user provided the number of nodes
if [ -z "$1" ]; then
  echo "Usage: $0 <number_of_nodes>"
  exit 1
fi

# Get the number of nodes from the command line argument
NUM_NODES=$1

# Base port
BASE_PORT=8000

# Directory to store the PIDs of the launched nodes
PID_DIR="./node_pids"
mkdir -p $PID_DIR

# Path to the node binary (update if necessary)
NODE_BINARY="./target/debug/untitled3"

# Generate the list of peer addresses
PEER_ADDRESSES=()
for (( i=0; i<NUM_NODES; i++ )); do
  port=$((BASE_PORT + i))
  PEER_ADDRESSES+=("127.0.0.1:$port")
done

# Function to launch a node
launch_node() {
  local node_id=$1
  local port=$((BASE_PORT + node_id))
  local address="127.0.0.1:$port"

  echo "Starting node $node_id on port $port..."

  # Prepare the peer addresses, excluding the node's own address
  local peers=$(printf ",%s" "${PEER_ADDRESSES[@]}")
  peers=${peers:1}
  peers=$(echo $peers | sed "s/$address,//")

  # Run the node and save its PID
  $NODE_BINARY --address "$address" --peers "$peers" > "$PID_DIR/node_$node_id.log" 2>&1 &
  echo $! > "$PID_DIR/node_$node_id.pid"
}

# Launch the specified number of nodes
for (( i=0; i<NUM_NODES; i++ )); do
  launch_node $i
done

echo "All $NUM_NODES nodes started."
