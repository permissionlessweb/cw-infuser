#!/bin/bash

# Run schema command for cw-ave and move the schema file
echo "Updating schema for cw-infuser..."

cargo schema -p cw-infusion-minter 
mv schema/cw-infuser.json ./contracts/cw-infuser/schema/

cargo schema -p cw721-svg  
mv schema/cw721-svg.json ./contracts/cw721/cw721-svg/schema/

cargo schema -p whitelist-mtree 
mv schema/whitelist-mtree.json ./contracts/whitelist/whitelist-merkletree/schema/


 
echo "Schema update completed."

## generate ts code
cd scripts/ts && yarn codegen