#!/bin/bash
# set -e

cd $(dirname "$0")
mkdir -p logs
DATE=$(date "+%Y_%m_%d")

cargo run --release near-social-api 2>&1 | tee -a logs/logs_$DATE.txt
