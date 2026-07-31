# `jboolean` must not be Rust's `bool` — proof of concept

For the change

```rust
-pub type jboolean = bool;      -pub const JNI_TRUE:  jboolean = true;   -pub const JNI_FALSE: jboolean = false;
+pub type jboolean = u8;        +pub const JNI_TRUE:  jboolean = 1;      +pub const JNI_FALSE: jboolean = 0;
```

## The symptom

```rust
if env.CallBooleanMethod0(obj, return_false) {
    // The Java method returned `false`. This block runs anyway.
}
```

`jni.h` declares `typedef unsigned char jboolean;`, and AAPCS64 defines only bits 0–7 of `x0` for a size-1
return — bits 8–63 are unspecified. So a Java `false` can arrive as `0x78352b8600`: low byte `0x00`,
correct, junk above.

Rust lowers an `extern` fn returning `bool` as `zeroext i1`, promising LLVM the *whole* register is 0 or 1.
Trusting that, LLVM compiles `if b { .. }` on arm64 to **no instruction at all** — the raw register is the
answer. The branch is taken.

**ART is conforming here; the `bool` typing is not.** Nothing in this report asks ART to change.

## Running it

```sh
./run.sh                                          # one attached device
./run.sh -H 127.0.0.1 -P 5035 -s emulator-5554    # device on another adb server
./static_evidence.sh                              # no device needed
```

Exit 0 means the bug reproduced. Needs `ANDROID_HOME`, an NDK, `javac`, `cargo-ndk`, a booted emulator, and
`rustup target add aarch64-linux-android x86_64-linux-android`.

`rust/probe/src/lib.rs` is built **twice from one source**, selected by feature: `published` links
jni-simple `=0.3.2` from crates.io (`jboolean = bool`), `local` links this working tree (`u8`). Nothing
else differs — same compiler, same flags, same Java object. Both builds export the same JNI symbol, so each
runs in its own process.

## The two checks

**[1] A Java `boolean` holding the byte `0x02`.** Written with `SetBooleanArrayRegion`, read back, widened
with `b as u32`. JNI defines only "zero is false, non-zero is true", so `0x02` is a legal value — but a
Rust `bool` may only ever widen to 0 or 1. **Always reproduces**, every config, both ABIs.

**[2] A Java method that returns `false`.** `returnFalse()` is `return false;` and `flagFalse` is never
assigned. Branch on each with `if b { 1 } else { 0 }`, which must yield 0. **Reproduces sometimes** — it
needs ART to leave junk in the upper bytes of the return register, which is an arm64 phenomenon and on
these emulators shows up under `-Xcheck:jni` on API 26 and 29. As
[the ART disassembly below](#why-mostly-under--xcheckjni) shows, CheckJNI is not the cause and the ordinary
path is not safe either.

## Output

```
jboolean PoC -- API 26, arm64-v8a

                                           crates.io 0.3.2      this branch
                                           (jboolean = bool)    (jboolean = u8)

  -- default --
  [1] boolean[0]=0x02  b as u32            2            BROKEN  2            ok
  [2] returnFalse()    CallBooleanMethod   0            ok      0            ok
  [2] returnFalse()    CallBooleanMethodA  0            ok      0            ok
  [2] flagFalse        GetBooleanField     0            ok      0            ok

  -- checkjni --
  [1] boolean[0]=0x02  b as u32            528642562    BROKEN  2            ok
  [2] returnFalse()    CallBooleanMethod   527681024    BROKEN  0            ok
  [2] returnFalse()    CallBooleanMethodA  0            ok      0            ok
  [2] flagFalse        GetBooleanField     0            ok      0            ok

  crates.io 0.3.2: 3 broken    this branch: 0 broken
  BUG DEMONSTRATED
```

`527681024` is what `if b { 1 } else { 0 }` returned for a method that returned `false`. The branch went
the wrong way, and the result is not even 0 or 1 — on arm64 that `if` compiles to nothing, so the raw
return register falls straight through.

Which check-[2] accessors fire varies between runs. On x86_64 only check [1] fires: 2 broken, both configs.
The SysV psABI defines only `%al` for a size-1 return, so LLVM emits `movzbl %al, %eax` for both typings
and junk above the low byte cannot reach the branch. That matches where the bug was hit: an arm64 device.

## Why mostly under `-Xcheck:jni`?

Not because CheckJNI is special. Both paths return an 8-byte `JValue` from an inner call in `x0` and then
declare their own return type as `jboolean` (`unsigned char`). AAPCS64 does not require the callee to
narrow that, so it comes down to whether the compiler happened to. From the API 26 emulator's own
`/system/lib64/libart.so`:

**`art::CheckJNI::CallBooleanMethod`** @ `0xf8a64` — returns `x0` verbatim:

```asm
f8b00:  bl   art::CheckJNI::CallMethodV(char const*, _JNIEnv*, ...)
f8b04:  ldr  x8, [x19, #0x28]     ; stack-guard check only
f8b0c:  cmp  x8, x9
f8b20:  ret                        ; x0 untouched -- all 64 bits leak
```

Note the leading `char const*` parameter: that is `__FUNCTION__`, the JNI function's own name. It is why
the junk observed on API 29 decodes as ASCII — `0x6874654d6e616501` is `01 'e' 'a' 'n' 'M' 'e' 't' 'h`, the
tail of `"CallBooleanMethod"`.

**`art::JNI::CallBooleanMethod`** @ `0x31a558` — the ordinary path:

```asm
31a80c:  bl   art::InvokeVirtualOrInterfaceWithVarArgs(...)
31a814:  mov  x19, x0              ; keep the whole JValue
...
31aab0:  mov  w0, w19              ; 32-bit move: clears bits 32..63
31aad0:  ret
```

That is the entire difference — one path got a 32-bit `mov`, the other did not.

**So the ordinary path is not safe either.** `mov w0, w19` narrows to 32 bits, not to 8. Bits 8–31 of the
`JValue` still flow into `x0`. They happened to be zero in these runs, which is why `default` looks clean;
nothing guarantees it. A `JValue` with junk in bits 8–31 leaks through the ordinary path too.

"Only under `-Xcheck:jni`" is therefore an accident of this ART build at this API level, not a property a
crate may rely on. API 35 is clean in both paths — a third accident. **ART is conforming in every one of
these cases**, because the ABI only ever defined the low byte.

To check this yourself:

```sh
adb pull /system/lib64/libart.so                     # /apex/com.android.art/lib64/ on newer APIs
llvm-nm --defined-only --demangle libart.so | grep 'CallBooleanMethod('
llvm-objdump -d --no-show-raw-insn --start-address=0xf8a64 --stop-address=0xf8b28 libart.so
```

## Why — `static_evidence.sh`

```asm
branch_bool:                    branch_u8:
    blr  x0                         blr  x0
    ret                             tst  w0, #0xff    ← low byte only
                                    cset w0, ne
                                    ret
```

`branch_bool` is `if f() { 1 } else { 0 }`. On arm64 it compiles to **nothing**: the raw return register is
the answer, so whatever ART left above the low byte decides the branch. Needs no device.

On x86_64 both typings emit `movzbl %al, %eax`, which is why check [2] cannot fire there.

## Notes

- The probe applies `black_box` to each result before testing it. Without that, LLVM folds the test away
  using the `range(i32 0, 2)` it believes the value has — the same defect propagating into the checking
  code.
- The `published` dependency is pinned `=0.3.2` so this keeps working after the fix is released.
- `run.sh` uses `app_process`, not `dalvikvm`: on API 29 `dalvikvm`'s `dlopen` cannot see `libdl.so` for a
  library under `/data/local/tmp`.
