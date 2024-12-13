#!/bin/bash

# List of Proxmox node IPs or hostnames
NODES=("10.8.10.250" "10.8.10.251" "10.8.10.252" "10.8.10.253" "10.8.10.254")
SSH_USER="root"
OUTPUT_DIR="./node_inventory"

# Ensure output directory exists
mkdir -p $OUTPUT_DIR

echo "Starting inventory of nodes..."
for NODE in "${NODES[@]}"; do
    echo "Collecting inventory from $NODE..."

    # Collect CPU, Memory, Disk, and Network Information
    ssh $SSH_USER@$NODE "bash -s" <<-'EOSSH' > "$NODE-inventory.log"
        echo "========== Node Information =========="
        echo "Hostname: $(hostname)"
        echo "IP Address: $(hostname -I | awk '{print $1}')"
        echo
        echo "========== CPU Information =========="
        lscpu
        echo
        echo "========== Memory Information =========="
        free -h
        echo
        echo "========== Storage Information =========="
        lsblk -o NAME,SIZE,TYPE,MOUNTPOINT
        echo
        echo "========== Disk Usage =========="
        df -h
        echo
        echo "========== Network Interfaces =========="
        ip -br addr show
EOSSH

    # Save inventory to file
    mv "$NODE-inventory.log" "$OUTPUT_DIR/$NODE-inventory.log"
    echo "Inventory for $NODE saved to $OUTPUT_DIR/$NODE-inventory.log"
done

echo "Inventory collection complete! Files saved in $OUTPUT_DIR"
