#!/bin/bash

for i in {1..100}; do
    printf -v num "%03d" "$i"  # Format number with leading zeros (e.g., 001, 042, 100)
    cat > "$i.json" <<EOF
{
    "name": "Infused Collage",
    "description": "IN1_${num}",
    "attributes": [],
    "image": "ipfs://Qme5Q1kcA2TcBbKjDDUE1gv3rkpeJ5NQppTamvagYwZMXe"
}
EOF