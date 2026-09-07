#!/bin/sh
SERVER="192.168.0.110"

RUST_FONTCONFIG_DLOPEN=1 cargo zigbuild --release --target armv7-unknown-linux-musleabihf && \
sshpass -p "" scp -P 2222 target/armv7-unknown-*/release/jobman root@$SERVER:/mnt/us/jobman 