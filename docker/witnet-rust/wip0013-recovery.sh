#!/usr/bin/env bash

WITNET_BINARY=${1:-./witnet}
WITNET_CONFIG_FILE=${2:-./witnet.toml}

RECOVERY_BANNER="

███████╗ ██████╗ ██████╗ ██╗  ██╗  ██╗ ██╗ ██╗
██╔════╝██╔═══██╗██╔══██╗██║ ██╔╝  ██║ ██║ ██║
█████╗  ██║   ██║██████╔╝█████╔╝   ██║ ██║ ██║
██╔══╝  ██║   ██║██╔══██╗██╔═██╗   ██║ ██║ ██║
██║     ╚██████╔╝██║  ██║██║  ██╗  ██║ ██║ ██║
╚═╝      ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝  ╚████████╔╝
██████╗ ███████╗ ██████╗ ██████╗     ╚█████╔╝
██╔══██╗██╔════╝██╔════╝██╔═══██╗     ╚██╔╝
██████╔╝█████╗  ██║     ██║   ██║      ██║
██╔══██╗██╔══╝  ██║     ██║   ██║      ██║
██║  ██║███████╗╚██████╗╚██████╔╝      ██║
╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═════╝       ██║
██╗   ██╗███████╗██████╗ ██╗   ██╗     ██║
██║   ██║██╔════╝██╔══██╗╚██╗ ██╔╝     ██║
██║   ██║█████╗  ██████╔╝ ╚████╔╝      ██║
╚██╗ ██╔╝██╔══╝  ██╔══██╗  ╚██╔╝       ██║
 ╚████╔╝ ███████╗██║  ██║   ██║        ██║
  ╚═══╝  ╚══════╝╚═╝  ╚═╝   ╚═╝        ╚═╝

╔══════════════════════════════════════════════════════════╗
║ LOCAL CHAIN IS FORKED. PROCEEDING TO AUTOMATIC RECOVERY. ║
╠══════════════════════════════════════════════════════════╣
║ This process will sanitize the local chain state by      ║
║ rewinding back to the point where the fork took place,   ║
║ and then continue synchronizing and operating as usual.  ║
╟──────────────────────────────────────────────────────────╢
║ This will take from 30 to 60 minutes depending on your   ║
║ network, CPU, RAM and hard disk speeds.                  ║
╟──────────────────────────────────────────────────────────╢
║ Learn more about why this recovery is needed:            ║
║ https://github.com/witnet/WIPs/blob/master/wip-0013.md   ║
╟──────────────────────────────────────────────────────────╢
║ Feel free to ask any questions on:                       ║
║ Discord:  https://discord.gg/X4uurfP                     ║
║ Telegram: https://t.me/witnetio                          ║
╚══════════════════════════════════════════════════════════╝
"

KNOWN_PEERS='\n    "46.4.102.43:22341",\n    "46.4.104.114:22339",\n    "49.12.120.229:22350",\n    "68.183.202.131:22339",\n    "78.46.66.69:22340",\n    "78.46.86.104:22342",\n    "78.46.86.104:22372",\n    "78.47.209.194:22339",\n    "81.30.157.7:22339",\n    "81.30.157.7:22345",\n    "81.30.157.7:22346",\n    "81.30.157.7:22348",\n    "81.30.157.7:22350",\n    "81.30.157.7:22351",\n    "81.30.157.7:22352",\n    "81.30.157.7:22353",\n    "85.114.132.66:22339",\n    "85.114.132.66:22348",\n    "85.114.132.66:22351",\n    "85.114.132.66:22352",\n    "88.198.8.177:22382",\n    "88.99.104.178:22376",\n    "94.130.165.149:22339",\n    "94.130.206.122:22349",\n    "94.130.30.59:22339",\n    "95.217.181.71:21337",\n    "104.218.233.118:18",\n    "116.202.116.196:22339",\n    "116.202.131.166:22339",\n    "116.202.131.24:22339",\n    "116.202.131.25:22339",\n    "116.202.131.30:22339",\n    "116.202.131.31:22339",\n    "116.202.131.32:22339",\n    "116.202.164.167:22346",\n    "116.202.164.173:22380",\n    "116.202.164.176:22339",\n    "116.202.218.95:22357",\n    "116.202.218.95:22376",\n    "116.202.35.247:22339",\n    "134.122.116.156:22339",\n    "135.181.0.31:22339",\n    "135.181.19.225:22343",\n    "135.181.194.161:21337",\n    "135.181.56.50:22342",\n    "135.181.60.149:22345",\n    "135.181.60.149:22350",\n    "135.181.60.184:22341",\n    "136.243.135.114:22339",\n    "136.243.93.124:22380",\n    "136.243.93.142:22352",\n    "136.243.93.142:22382",\n    "136.243.94.114:22353",\n    "136.243.94.171:22346",\n    "136.243.94.30:22350",\n    "136.243.95.38:22342",\n    "136.243.95.38:22348",\n    "138.201.241.61:22365",\n    "138.201.66.37:22348",\n    "138.201.83.20:22339",\n    "138.201.83.56:22340",\n    "144.91.113.168:21337",\n    "148.251.127.248:22341",\n    "148.251.128.18:22348",\n    "148.251.128.19:22342",\n    "148.251.128.26:22341",\n    "157.245.171.146:21337",\n    "159.69.139.239:22339",\n    "159.69.56.28:22339",\n    "159.69.56.28:22388",\n    "159.69.72.123:22339",\n    "159.69.74.122:22348",\n    "159.69.74.123:22366",\n    "159.69.74.79:22346",\n    "159.69.74.96:22339",\n    "161.35.167.68:22339",\n    "167.172.29.131:22339",\n    "168.119.5.23:22360",\n    "168.119.5.26:22353",\n    "176.9.29.25:22342",\n    "176.9.66.252:22383",\n    "188.40.131.24:22339",\n    "188.40.94.105:22339",\n    "192.241.148.38:22339",\n    "195.201.167.113:22339",\n    "195.201.173.77:22339",\n    "195.201.181.221:22339",\n    "195.201.181.245:22339",\n    "195.201.240.189:22339",\n    "213.239.234.132:22348",\n'

# Just a pretty logging helper
function log {
  echo "[WIP0013_RECOVERY] $1"
}

# A helper for calculating ETAs
function eta {
  START=$1
  NOW=$2
  PROGRESS=$3
  if [ "$PROGRESS" == "00" ]; then
    echo "will be shown as synchronization moves forward..."
  else
    ELAPSED=$(( NOW - START ))
    SPEED=$((PROGRESS * 1000 / ELAPSED))
    if [ "$SPEED" == "0" ]; then
        SPEED=1
    fi
    REMAINING_PROGRESS=$(( 10000 - PROGRESS ))
    REMAINING_TIME=$((REMAINING_PROGRESS * 1000 / SPEED ))
    echo $(( REMAINING_TIME / 60 )) minutes $((REMAINING_TIME % 60)) seconds
  fi
}

# This script can be skipped by setting environment variable SKIP_WIP0013_RECOVERY to "true"
if [[ "$SKIP_WIP0013_RECOVERY" == "true" ]]; then
  log "Skipping WIP-0013 recovery"
  exit 0
fi

# Make sure the arguments make sense
if ! command -v "$WITNET_BINARY" &> /dev/null; then
  log "ERROR: The provided witnet binary (first argument to this script) is not a valid executable file: $WITNET_BINARY"
  exit 1
fi
if [ ! -f "$WITNET_CONFIG_FILE" ]; then
  log "ERROR: The provided witnet configuration file (second argument to this script) is not a valid configuration file: $WITNET_CONFIG_FILE"
  exit 2
fi

# Read configuration (e.g. node server address) from config file
log "Using configuration file at $WITNET_CONFIG_FILE"
HOST=$(grep "server_addr" "$WITNET_CONFIG_FILE" | sed -En "s/server_addr = \"(.*)\"/\1/p" | sed -E "s/0\.0\.0.\0/127.0.0.1/g" )
ADDRESS=$(echo "$HOST" | cut -d':' -f1)
PORT=$(echo "$HOST" | cut -d':' -f2)

# Check connection to local witnet node
TIME_TO_NEXT_RETRY=5
log "Checking connection to local witnet node at $HOST"
while true
  if nc -zv "$ADDRESS" "$PORT" &>/dev/null; then
    log "Successful connection to local witnet node at $HOST"
    break
  else
    log "ERROR: Failed to connect to local witnet node at $HOST"
    log "Retrying in $TIME_TO_NEXT_RETRY seconds"
    sleep "$TIME_TO_NEXT_RETRY"
    TIME_TO_NEXT_RETRY=$(( 2 * TIME_TO_NEXT_RETRY ))
  fi
do true; done

# Check whether the local witnet node is below WIP-0013 "common checkpoint" (#364269)
if ! $WITNET_BINARY node blockchain --epoch 364269 --limit 1 2>&1 | grep -q "block for epoch"; then
  log "The local witnet node at $HOST seems to be syncing blocks prior to the WIP-0013 fork. No recovery action is needed"
  exit 0
fi

# Check whether the local witnet node is on the `A` chain, and if so, skip recovery
if $WITNET_BINARY node blockchain --epoch 368699 --limit 2 2>&1 | grep -q '#368699 had digest 3b0b03df'; then
  log "The local witnet node at $HOST seems to be on the leading chain. No recovery action is needed"
  exit 0
fi

# There is no way back, recovery is needed
echo "$RECOVERY_BANNER"

# Update known peers in configuration file
log "Updating known_peers in configuration file at $WITNET_CONFIG_FILE"
sed -ziE "s/known_peers\s*=\s*\[\n.*\\,\n\]/known_peers = [$KNOWN_PEERS]/g"  "$WITNET_CONFIG_FILE" &&
log "Successfully updated known_peers in configuration file" ||
log "ERROR: Failed to update known_peers in configuration file at $WITNET_CONFIG_FILE"

# Rewind local chain back to the WIP-0013 "common checkpoint" (#364269)
log "Triggering rewind of local block chain back to epoch #364269"
$WITNET_BINARY node rewind --epoch 364269 &>/dev/null
REWIND_START=$(date +%s)

# Flush existing peers and inject new peers in runtime
$WITNET_BINARY node clearPeers 2>&1 | grep -q "Successful" &&
log "Successfully cleared existing peers from buckets" ||
log "ERROR: Failed to clear existing peers from buckets"
echo "$KNOWN_PEERS" | sed -r "s/\\\n\s*\"([0-9]+\.[0-9]+\.[0-9]+\.[0-9]+\:[0-9]+)\"\,/\1, /g" | "$WITNET_BINARY" node addPeers 2>&1 | grep -q "Successful" &&
log "Successfully added healthy peers" ||
log "ERROR: Failed to add new list of helthy peers"

# Wait for the rewind to complete, showing progress and ETA
while true
  STATS=$($WITNET_BINARY node nodeStats 2>&1)
  if echo "$STATS" | grep -q "synchronized"; then
    log "Successfully finished rewinding and synchronizing!"
    break
  else
    NOW=$(date +%s)
    PERCENTAGE=$(echo "$STATS" | sed -En "s/.*\:\s*(.*)\.(.*)\%.*/\1.\2%/p")
    PERCENTAGE_RAW=$(echo "$PERCENTAGE" | sed -En "s/0*(.*)\.(.*)\%/\1\2/p")
    log "Still rewinding and synchronizing. Progress: $PERCENTAGE. ETA: $(eta "$REWIND_START" "$NOW" "$PERCENTAGE_RAW")"
    sleep 30
  fi
do true; done