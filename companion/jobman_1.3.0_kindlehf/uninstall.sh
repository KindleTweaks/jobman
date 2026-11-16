#!/bin/sh

rm -rf /mnt/us/jobman
rm -f /mnt/us/documents/jobman.sh

if [ -f /etc/upstart/jobman_recovery.conf ]; then
    mntroot rw

    if ! grep -q "start on started system_setup" /etc/upstart/system_setup_after_shpm.conf; then
        sed -i "/#This upstart node handles configuring of system after coming out of shipment mode/a start on started system_setup" /etc/upstart/system_setup_after_shpm.conf
    fi

    rm -f /etc/upstart/jobman_recovery.conf
    mntroot ro
fi