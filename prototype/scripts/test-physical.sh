#!/usr/bin/env bash
# Local pre-release check for a real physical keyboard path.
# Not used in CI: /dev/uinput is not available to hosted runners.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
export WAYLAND_DISPLAY="${WAYLAND_DISPLAY:-wayland-1}"

command -v cosmic-edit >/dev/null || { echo "cosmic-edit not found" >&2; exit 2; }
command -v gcc >/dev/null || { echo "gcc not found" >&2; exit 2; }
[ -w /dev/uinput ] || { echo "/dev/uinput is not writable" >&2; exit 2; }

STATE_FILE="$XDG_RUNTIME_DIR/rpci/state"
CONTROL_SOCK="$XDG_RUNTIME_DIR/rpci/control.sock"
[ -S "$CONTROL_SOCK" ] || { echo "IME control socket not found" >&2; exit 2; }

ORIGINAL_STATE="$(cat "$STATE_FILE" 2>/dev/null || true)"
set_state() {
    python3 - "$CONTROL_SOCK" "$1" <<'PY'
import socket, sys
sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
sock.connect(sys.argv[1])
sock.sendall(sys.argv[2].encode())
sock.close()
PY
}
if [ "$ORIGINAL_STATE" != "中 五" ]; then
    set_state "toggle"
    sleep 1
fi

TMP="$(mktemp -d)"
EDITOR_PID=""
cleanup() {
    if [ -n "$EDITOR_PID" ]; then
        kill "$EDITOR_PID" 2>/dev/null || true
        wait "$EDITOR_PID" 2>/dev/null || true
    fi
    if [ -n "${TEST_FILE:-}" ]; then
        pkill -f "$TEST_FILE" 2>/dev/null || true
    fi
    if [ -n "$ORIGINAL_STATE" ] && [ "$(cat "$STATE_FILE" 2>/dev/null || true)" != "$ORIGINAL_STATE" ]; then
        set_state "toggle" || true
    fi
    rm -rf "$TMP"
}
trap cleanup EXIT

cat > "$TMP/uinput-key.c" <<'C'
#include <linux/uinput.h>
#include <linux/input.h>
#include <fcntl.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <sys/ioctl.h>

static int fd;

static void key(int code) {
    struct input_event event = {0};
    event.type = EV_KEY;
    event.code = code;
    event.value = 1;
    (void)!write(fd, &event, sizeof event);
    event.value = 0;
    (void)!write(fd, &event, sizeof event);
}

int main(void) {
    fd = open("/dev/uinput", O_WRONLY | O_NONBLOCK);
    if (fd < 0) return 1;
    ioctl(fd, UI_SET_EVBIT, EV_KEY);
    for (int i = 1; i <= 255; i++) ioctl(fd, UI_SET_KEYBIT, i);
    struct uinput_setup setup = {0};
    snprintf(setup.name, UINPUT_MAX_NAME_SIZE, "rpci-physical-test");
    setup.id.bustype = BUS_USB;
    setup.id.vendor = 1;
    setup.id.product = 2;
    setup.id.version = 1;
    ioctl(fd, UI_DEV_SETUP, &setup);
    ioctl(fd, UI_DEV_CREATE);
    sleep(1);

    int wubi[] = {17, 16, 23, 21, 57}; /* w q i y space */
    for (unsigned i = 0; i < sizeof wubi / sizeof wubi[0]; i++) {
        key(wubi[i]);
        usleep(100000);
    }
    sleep(1);

    struct input_event event = {0};
    event.type = EV_KEY;
    event.code = 29; /* left ctrl */
    event.value = 1;
    (void)!write(fd, &event, sizeof event);
    event.code = 57; /* space */
    (void)!write(fd, &event, sizeof event);
    usleep(100000);
    event.value = 0;
    (void)!write(fd, &event, sizeof event);
    event.code = 29;
    (void)!write(fd, &event, sizeof event);
    usleep(300000);

    int abc[] = {30, 48, 46};
    for (unsigned i = 0; i < sizeof abc / sizeof abc[0]; i++) {
        key(abc[i]);
        usleep(150000);
    }
    sleep(1);

    event.type = EV_KEY;
    event.code = 29;
    event.value = 1;
    (void)!write(fd, &event, sizeof event);
    event.code = 31; /* s */
    (void)!write(fd, &event, sizeof event);
    usleep(100000);
    event.value = 0;
    (void)!write(fd, &event, sizeof event);
    event.code = 29;
    (void)!write(fd, &event, sizeof event);

    usleep(500000);
    ioctl(fd, UI_DEV_DESTROY);
    close(fd);
    return 0;
}
C
gcc -O2 -o "$TMP/uinput-key" "$TMP/uinput-key.c"

FOCUS="$ROOT/target/debug/focus"
if [ ! -x "$FOCUS" ]; then
    cargo build --quiet -p ime-probe --bin focus
fi
[ -x "$FOCUS" ] || { echo "focus helper missing" >&2; exit 2; }

TEST_FILE="$TMP/physical.txt"
cosmic-edit "$TEST_FILE" >/dev/null 2>&1 &
EDITOR_PID=$!
FOCUS_OUT=""
for _ in 1 2 3 4 5; do
    sleep 1
    FOCUS_OUT="$("$FOCUS" "$(basename "$TEST_FILE")" 2>&1 || true)"
    case "$FOCUS_OUT" in
        *"$(basename "$TEST_FILE")"*) break ;;
    esac
done
case "$FOCUS_OUT" in
    *"$(basename "$TEST_FILE")"*) ;;
    *)
        echo "could not focus cosmic-edit window for $TEST_FILE" >&2
        printf '%s\n' "$FOCUS_OUT" >&2
        exit 1
        ;;
esac

"$TMP/uinput-key"
sleep 1
if ! grep -q '你abc' "$TEST_FILE"; then
    echo "physical keyboard check failed" >&2
    od -An -c "$TEST_FILE" >&2 || true
    exit 1
fi

echo "physical keyboard check passed: 你abc"
