#!/usr/bin/env bash
#
# Builds one Rust probe twice -- against jni-simple from crates.io
# (jboolean = bool) and against this working tree (jboolean = u8) -- and runs
# both on an Android device against the same Java object.
#
# Exit: 0 = bug reproduced, 1 = both builds agreed, 2 = setup failure.
#
set -u -o pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT="$HERE/out"
ANDROID_HOME="${ANDROID_HOME:-$HOME/Library/Android/sdk}"
NDK_VERSION="${NDK_VERSION:-27.1.12297006}"
BUILD_TOOLS="${BUILD_TOOLS:-34.0.0}"
MIN_API=26
DEVDIR=/data/local/tmp/jboolmre
SERIAL="${ANDROID_SERIAL:-}"

usage() {
    cat <<EOF
usage: run.sh [-s SERIAL] [-H HOST] [-P PORT] [-q]

  -s SERIAL   target device (default: the only attached one)
  -H, -P      adb server host/port, for a device on another adb server
  -q          skip the build, reuse out/
EOF
}

die() { printf 'run.sh: %s\n' "$*" >&2; exit 2; }

QUICK=0
while [ $# -gt 0 ]; do
    case "$1" in
        -s) SERIAL="${2:-}"; shift 2 ;;
        -H) export ANDROID_ADB_SERVER_ADDRESS="${2:-}"; shift 2 ;;
        -P) export ANDROID_ADB_SERVER_PORT="${2:-}"; shift 2 ;;
        -q) QUICK=1; shift ;;
        -h|--help) usage; exit 0 ;;
        *) die "unknown option '$1'" ;;
    esac
done

ADB="$ANDROID_HOME/platform-tools/adb"
D8="$ANDROID_HOME/build-tools/$BUILD_TOOLS/d8"
[ -x "$ADB" ] || die "adb not found at $ADB"
[ -x "$D8" ] || die "d8 not found at $D8 (set BUILD_TOOLS=)"
[ -d "$ANDROID_HOME/ndk/$NDK_VERSION" ] || die "no NDK $NDK_VERSION (set NDK_VERSION=)"
command -v javac >/dev/null || die "javac not on PATH"
command -v cargo-ndk >/dev/null || die "cargo-ndk not installed"

if [ -z "$SERIAL" ]; then
    SERIAL="$("$ADB" devices | awk '$2 == "device" { print $1 }')"
    [ -n "$SERIAL" ] || die "no device attached"
    [ "$(printf '%s\n' "$SERIAL" | wc -l)" -eq 1 ] || die "several devices attached; use -s (and -H/-P)"
fi

API="$("$ADB" -s "$SERIAL" shell getprop ro.build.version.sdk | tr -d '\r')"
ABI="$("$ADB" -s "$SERIAL" shell getprop ro.product.cpu.abi | tr -d '\r')"
[ -n "$API" ] || die "cannot reach device '$SERIAL'"

# ---------------------------------------------------------------------- build --

LIBDIR="$OUT/lib/$ABI"

if [ "$QUICK" -eq 0 ]; then
    mkdir -p "$OUT/classes" "$LIBDIR"
    javac --release 8 -nowarn -d "$OUT/classes" "$HERE"/java/*.java || die "javac failed"
    # Any android.jar serves as the --lib stub; only --min-api matters.
    jar="$(ls -1 "$ANDROID_HOME"/platforms/android-*/android.jar | tail -1)"
    "$D8" --release --min-api "$MIN_API" --lib "$jar" --output "$OUT" "$OUT"/classes/*.class || die "d8 failed"

    for v in published local; do
        ( cd "$HERE/rust/probe" && \
          ANDROID_NDK_HOME="$ANDROID_HOME/ndk/$NDK_VERSION" CARGO_TARGET_DIR="$OUT/cargo-$v" \
          cargo ndk -t "$ABI" -p "$MIN_API" -o "$OUT/stage-$v" build --release --features "$v" ) \
            >/dev/null 2>&1 || die "cargo ndk failed for '$v'"
        cp "$OUT/stage-$v/$ABI/libjboolean_probe.so" "$LIBDIR/libprobe_$v.so"
    done
fi

# Libraries are staged per ABI, so -q against a different device is caught here
# instead of silently loading the wrong .so and reporting a false negative.
[ -f "$LIBDIR/libprobe_published.so" ] || die "no $ABI build in out/; drop -q to build it"

# ----------------------------------------------------------------------- run --

"$ADB" -s "$SERIAL" shell "rm -rf $DEVDIR && mkdir -p $DEVDIR" || die "cannot write $DEVDIR"
"$ADB" -s "$SERIAL" push "$OUT/classes.dex" "$LIBDIR"/*.so "$DEVDIR/" >/dev/null 2>&1 || die "adb push failed"

# app_process rather than dalvikvm: on API 29 dalvikvm's dlopen cannot see
# libdl.so for a library under /data/local/tmp.
run() { # <variant> <extra flags>
    "$ADB" -s "$SERIAL" shell \
        "ANDROID_DATA=$DEVDIR LD_LIBRARY_PATH=$DEVDIR CLASSPATH=$DEVDIR/classes.dex \
         app_process ${2:-} -Djbool.lib=$DEVDIR/libprobe_$1.so $DEVDIR JBoolMre" 2>&1 | tr -d '\r'
}

printf '\njboolean PoC -- API %s, %s\n\n' "$API" "$ABI"
printf '  %-40s %-20s %s\n' "" "crates.io 0.3.2" "this branch"
printf '  %-40s %-20s %s\n' "" "(jboolean = bool)" "(jboolean = u8)"

broken=0
for cfg in default checkjni; do
    [ "$cfg" = checkjni ] && flags=-Xcheck:jni || flags=
    pub="$(run published "$flags")"
    loc="$(run local "$flags")"
    # Insist on a well-formed result rather than treating "no BROKEN" as a pass:
    # a failed System.load would otherwise read as the bug not reproducing.
    # Counted rather than hard-coded, so adding a check needs no edit here.
    n_pub=$(printf '%s\n' "$pub" | grep -cE '\|(ok|BROKEN)$')
    n_loc=$(printf '%s\n' "$loc" | grep -cE '\|(ok|BROKEN)$')
    [ "$n_pub" -gt 0 ] && [ "$n_pub" -eq "$n_loc" ] || die "probe did not run under '$cfg':
$pub
---
$loc"

    printf '\n  -- %s --\n' "$cfg"
    # Both runs emit the same checks in the same order, so pair them line by line.
    paste -d'|' <(printf '%s\n' "$pub") <(printf '%s\n' "$loc") \
        | awk -F'|' '{ printf "  %-40s %-12s %-7s %-12s %s\n", $1, $2, $3, $5, $6 }'

    broken=$((broken + $(printf '%s\n' "$pub" | grep -c BROKEN)))
    printf '%s\n' "$loc" | grep -q BROKEN && die "the local build reported a failure under '$cfg'"
done

"$ADB" -s "$SERIAL" shell "rm -rf $DEVDIR" >/dev/null 2>&1

printf '\n  crates.io 0.3.2: %d broken    this branch: 0 broken\n' "$broken"
if [ "$broken" -gt 0 ]; then
    printf '  BUG DEMONSTRATED -- same source, same flags, same Java object.\n'
    printf '  The only difference is which jni-simple was linked.\n\n'
    exit 0
fi

cat <<'EOF'
  NOT REPRODUCED on this device.

  Check [1] should fire everywhere, so a clean run here suggests something is
  wrong with the setup rather than with the typing. Check ./static_evidence.sh
  and ../tests/jboolean_dirty_register.rs, neither of which depends on ART.

EOF
exit 1
