#!/bin/bash
cd /home/wwwroot/bag/industrial-protocols-rust

CRATES=(
  industrial-protocols-opcua
  industrial-protocols-bacnet
  industrial-protocols-ethernet-ip
  industrial-protocols-profinet
  industrial-protocols-mqtt
  industrial-protocols-dnp3
  industrial-protocols-hart
  industrial-protocols-cclink
  industrial-protocols-iec61850
  industrial-protocols-canopen
  industrial-protocols-cclinkie
  industrial-protocols-hartip
  industrial-protocols-lin
  industrial-protocols-kline
  industrial-protocols-dali
  industrial-protocols-saej1850
  industrial-protocols-ethercat
  industrial-protocols-powerlink
  industrial-protocols-sercos
  industrial-protocols-profibus
  industrial-protocols-devicenet
  industrial-protocols-foundationfieldbus
  industrial-protocols-asinterface
  industrial-protocols-iolink
  industrial-protocols-flexray
  industrial-protocols-most
  industrial-protocols-lonworks
  industrial-protocols-controlnet
  industrial-protocols-interbus
  industrial-protocols-worldfip
  industrial-protocols-lightbus
  industrial-protocols-modbusplus
  industrial-protocols-isa100
  industrial-protocols-wirelesshart
  industrial-protocols-pci
  industrial-protocols-vme
  industrial-protocols-cpci
)

LOG="publish.log"
TOTAL=${#CRATES[@]}
echo "=== Starting publish of $TOTAL crates at $(date) ===" | tee -a "$LOG"

i=0
while [ $i -lt $TOTAL ]; do
  p="${CRATES[$i]}"
  idx=$((i + 1))
  echo "[$(date)] [$idx/$TOTAL] $p ..." | tee -a "$LOG"
  
  # Remove stale lock file
  rm -f target/package/.cargo-lock 2>/dev/null
  
  OUTPUT=$(cargo publish -p "$p" --allow-dirty 2>&1)
  rc=$?
  
  if [ $rc -eq 0 ]; then
    echo "[$(date)] [$idx/$TOTAL] OK: $p" | tee -a "$LOG"
    ((i++))
  elif echo "$OUTPUT" | grep -q "429"; then
    # Rate limited — wait 11 minutes and retry same crate
    WAIT_TIME=$(echo "$OUTPUT" | grep -oP 'after \K[^ ]+ [^ ]+' | head -1)
    echo "[$(date)] [$idx/$TOTAL] RATE LIMITED, waiting 11 min, retrying $p" | tee -a "$LOG"
    # Save full output for debugging
    echo "$OUTPUT" >> "$LOG"
    sleep 660
  else
    # Other error — save output and continue
    echo "[$(date)] [$idx/$TOTAL] ERROR on $p (rc=$rc):" | tee -a "$LOG"
    echo "$OUTPUT" >> "$LOG"
    ((i++))
  fi
done
echo "=== Done at $(date) ===" | tee -a "$LOG"
