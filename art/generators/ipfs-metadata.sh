#!/bin/bash

## todo: 
## - add number to make
## - prompt json metadata file type
## - prompt description prefix
## - prompt path to nft file attributes 
imgs=("ipfs://QmQFQMeDMUMe2wCwm2MwrXBeN88GZ1qgsDx7tTBCvvZbXB")
title="EXPLORE"
max_supply="22"

for (( i=1; i<=${max_supply}; i++ )); do
  printf -v num "%03d" "$i"
  img=${imgs[$RANDOM % ${#imgs[@]}]}
  cat > "output/$i.json" <<EOF
{
  "name": "${title}",
  "description": "Edition - ${num}/${max_supply}",
  "attributes": [
    {
      "trait_type": "Artist",
      "value": "Hard-Nett"
    }
  ],
  "image": "$img"
}
EOF
done