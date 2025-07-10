#!/bin/bash

# Run schema command for cw-ave and move the schema file
echo "Updating schema for cw-infuser..."
cargo schema --package cw-infuser
mv schema/cw-infuser.json ./contracts/cw-infuser/schema/

 
echo "Schema update completed."

## generate ts code
cd scripts/ts && yarn codegen