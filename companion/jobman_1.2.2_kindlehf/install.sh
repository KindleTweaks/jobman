#!/bin/sh

if cat /proc/cmdline | grep androidboot.veritymode; then
    echo "Cannot Install on Rootless!"
    exit 1
fi

echo "Copying Jobman Folder..."

if [ -f /mnt/us/jobman/ssh/etc/config.toml ]; then
    cp -f /mnt/us/jobman/ssh/etc/config.toml /mnt/us/config.toml.bak
fi

if [ -f /mnt/us/jobman/ssh/etc/dropbear/authorized_keys ]; then
    cp -f /mnt/us/jobman/ssh/etc/dropbear/authorized_keys /mnt/us/authorized_keys.bak
fi

cp -rf ./jobman/* /mnt/us 
rm -rf ./jobman

if [ -f /mnt/us/config.toml.bak ]; then
    cp -f /mnt/us/config.toml.bak /mnt/us/jobman/ssh/etc/config.toml
    rm -f /mnt/us/config.toml.bak
fi

if [ -f /mnt/us/authorized_keys.bak ]; then
    cp -f /mnt/us/authorized_keys.bak /mnt/us/jobman/ssh/etc/dropbear/authorized_keys
    rm -f /mnt/us/authorized_keys.bak
fi

# Handle Recovery Updates! (*important*)
if [ -f /etc/upstart/jobman_recovery.conf ] && [ -f /mnt/us/jobman/jobman_recovery.conf ]; then
    mntroot rw
    rm -f /etc/upstart/jobman_recovery.conf
    cp -f /mnt/us/jobman/jobman_recovery.conf /etc/upstart/jobman_recovery.conf
    mntroot ro
fi