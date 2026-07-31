#!/usr/bin/env bash
#
# Why the bug exists, straight from rustc. No device, no emulator, no ART.
#
set -eu -o pipefail

OUT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/out/evidence"
mkdir -p "$OUT"

cat > "$OUT/abi.rs" <<'EOF'
// The two ways to declare a JNI function returning `jboolean`.
// jni.h says:  typedef unsigned char jboolean;
type AsBool = extern "system" fn() -> bool;
type AsU8   = extern "system" fn() -> u8;

// `if env.someMethodReturningFalse() { .. }`
#[no_mangle] pub unsafe extern "C" fn branch_bool(f: AsBool) -> u32 { if f() { 1 } else { 0 } }
#[no_mangle] pub unsafe extern "C" fn branch_u8(f: AsU8) -> u32 { if f() != 0 { 1 } else { 0 } }
EOF

for target in aarch64-linux-android x86_64-linux-android; do
    if ! rustup target list --installed 2>/dev/null | grep -qx "$target"; then
        printf '\n%s: skipped (rustup target add %s)\n' "$target" "$target"
        continue
    fi

    rustc --crate-type=lib --edition 2021 -O --target "$target" \
          --emit=llvm-ir,asm -o "$OUT/abi-$target.ll" "$OUT/abi.rs" 2>/dev/null
    asm="$OUT/abi-$target.s"

    printf '\n=== %s ===\n\n' "$target"
    grep -E 'tail call' "$OUT/abi-$target.ll" | sed -e 's/^ */  /' -e 's/ #[0-9]*$//'
    printf '\n'
    awk '/^branch_(bool|u8):/ { f = 1; print "  " $0; next }
         f && /^\.Lfunc_end/  { f = 0; print ""; next }
         f && $0 !~ /^[[:space:]]*\./ && NF { print "     " $0 }' "$asm"
done

cat <<'EOF'
  `zeroext i1` promises the ABI that the whole return register is exactly 0
  or 1. jni.h promises only the low byte, so it is not the JVM's to keep.

  On arm64 that promise costs the branch entirely: `if f() {1} else {0}`
  compiles to `blr x0; ret`, so whatever ART left in the upper bytes of x0
  decides it. The `u8` version tests `w0 & 0xff`, which is all JNI defines.

  x86_64 masks with `movzbl` either way, so the branch is safe there; the
  damage shows up instead as a value that is neither 0 nor 1 -- check [1].
EOF
