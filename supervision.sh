#!/bin/bash

# Set default values
DEFAULT_RPC_URL="https://mainnet.helius-rpc.com/?api-key=59c981e3-3dc1-4c64-9b5a-2c315893fdcc"
DEFAULT_KEY="~/.config/solana/id2.json"
DEFAULT_FEE=9500000
DEFAULT_THREADS=4

# Assign arguments with defaults
RPC_URL=${1:-$DEFAULT_RPC_URL}
KEY=${2:-$DEFAULT_KEY}
FEE=${3:-$DEFAULT_FEE}
THREADS=${4:-$DEFAULT_THREADS}

# Command and its arguments, with dynamic values
COMMAND="./target/release/ore --rpc ${RPC_URL} --keypair ${KEY} --priority-fee ${FEE} mine --threads ${THREADS}"

# Loop indefinitely
while true; do
  echo "Starting the process..."
  
  # Execute the command
  eval $COMMAND
  
  # If the command was successful, exit the loop
  # Remove this if you always want to restart regardless of exit status
  [ $? -eq 0 ] && break
  
  echo "Process exited with an error. Restarting in 5 seconds..."
  sleep 5
done