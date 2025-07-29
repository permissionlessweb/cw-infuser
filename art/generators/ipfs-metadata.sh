#!/bin/bash

## todo: 
## - add number to make
## - prompt json metadata file type
## - prompt description prefix
## - prompt path to nft file attributes 

for i in {1..22}; do
    printf -v num "%03d" "$i"  # Format number with leading zeros (e.g., 001, 042, 100)
    cat > "$i.json" <<EOF
{
    "name": "Infused Collage",
    "description": "INFC004_${num}",
    "attributes": [],
    "image": "ipfs://QmbbcwQru4CuoKGTj59BcRRS9kho9HfWKwQSa4ZeyKiL6Z"
}
EOF
done
