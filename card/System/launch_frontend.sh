#!/bin/sh
SD="${SLOT_B_ROOT:-/mnt/sdcard}"
SYS="$SD/System"
RUN="${AGS_RUN:-/run}"
export SLOT_B_ROOT="$SD"

log() {
	echo "$(date '+%H:%M:%S') $*" >> "$SD/slot-b-session.log" 2>/dev/null
}

pid="$(cat "$RUN/slot-b-led.pid" 2>/dev/null)"
if [ -z "$pid" ] || [ ! -d "/proc/$pid" ]; then
	sh "$SYS/led.sh" once
	sh "$SYS/led.sh" loop > /dev/null 2>&1 &
	echo $! > "$RUN/slot-b-led.pid"
fi

echo "|/bin/sh $SYS/coresave.sh %p %s %e %t" > /proc/sys/kernel/core_pattern 2>/dev/null

if [ -f "$SD/RUN_PROBE" ]; then
	rm -f "$SD/RUN_PROBE"
	log "probe $(AGS_PROBE_NET_WAIT_S=15 sh "$SYS/probe.sh" 2>&1)"
fi

[ -f "$SD/slot-b.log" ] && mv -f "$SD/slot-b.log" "$SD/slot-b.log.1"
log "exec $SYS/slot-b"

exec /lib/ld-linux-aarch64.so.1 "$SYS/slot-b" > "$SD/slot-b.log" 2>&1
