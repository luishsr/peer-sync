#!/bin/bash

# Directory containing the PIDs of the launched nodes
PID_DIR="./node_pids"

# Function to stop a node
stop_node() {
  local pid_file=$1

  if [ -f $pid_file ]; then
    local pid=$(cat $pid_file)
    echo "Stopping node with PID $pid..."
    kill $pid
    rm $pid_file
  else
    echo "PID file $pid_file not found."
  fi
}

# Stop all nodes
for pid_file in $PID_DIR/*.pid; do
  stop_node $pid_file
done

echo "All nodes stopped."
