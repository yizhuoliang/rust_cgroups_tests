#!/bin/bash

# Update the package lists
apt-get update

# Install sudo if it's not installed
apt-get install -y sudo
sudo apt-get install -y vim

# Remount /sys/fs/cgroup with read-write permissions
sudo mount -o remount,rw /sys/fs/cgroup

# Clone the GitHub repository
git clone https://github.com/yizhuoliang/rust_cgroups_tests.git

# Change directory to the cloned repository
cd rust_cgroups_tests

# Install cargo-edit if not already installed
cargo install cargo-edit

# Add the gettid dependency to your Cargo.toml
cargo add gettid
