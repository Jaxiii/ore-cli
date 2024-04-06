#!/bin/bash

while true
do
  echo "Running"
   ./target/release/ore --rpc "http://167.71.140.228/v1/rpc" --keypair ./id.json --priority-fee 300000 mine --threads 8
  echo "Exited"
done