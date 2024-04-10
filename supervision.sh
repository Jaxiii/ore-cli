#!/bin/bash

# Set default values
DEFAULT_RPC_URL="https://api.mainnet-beta.solana.com"
DEFAULT_KEY="~/.config/solana/id.json"
DEFAULT_FEE=95000
DEFAULT_JITO=10000
DEFAULT_THREADS=4


# Assign arguments with defaults
RPC_URL=${1:-$DEFAULT_RPC_URL}
KEY=${2:-$DEFAULT_KEY}
FEE=${3:-$DEFAULT_FEE}
JITO_FEE=${4:-DEFAULT_JITO}
THREADS=${5:-$DEFAULT_THREADS}

COMMAND= ./target/release/ore --rpc $RPC_URL --jito-client "https://mainnet.block-engine.jito.wtf/api/v1/transactions" --keypair $KEY --priority-fee $FEE --jito-enable --jito-fee $JITO_FEE mine --threads $THREADS

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
