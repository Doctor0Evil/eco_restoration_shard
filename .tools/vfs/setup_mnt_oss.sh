#!/bin/bash
# FILE .tools/vfs/setup_mnt_oss.sh

# Create VFS root owned by the current user
sudo mkdir -p /mnt/oss
sudo chown "$USER":"$USER" /mnt/oss

# Core logical layout for cc-vfs
mkdir -p /mnt/oss/{repos,shards,index,metrics,staging}

# Bind-mount the eco_restoration_shard repo into the VFS view
# (adjust path to your checkout)
sudo mkdir -p /mnt/oss/repos/eco_restoration_shard
sudo mount --bind "$HOME/code/eco_restoration_shard" \
    /mnt/oss/repos/eco_restoration_shard

# Research / PROD shard lanes
mkdir -p /mnt/oss/shards/{eco_research,eco_prod}

# Metrics and index trees (for KER, ecosafety, cc-path indices)
mkdir -p /mnt/oss/metrics/{storage,compute}
mkdir -p /mnt/oss/index
mkdir -p /mnt/oss/staging
