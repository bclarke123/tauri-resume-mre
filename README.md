# tauri-resume-mre

Minimal reproduction for a mobile-only deadlock/panic in `tauri-runtime-wry`:
the `Resumed`/`Suspended` event branch holds a `windows.0` `RefCell` borrow across
the user `RunEvent` callback, so mutating the window set from that callback re-enters
`windows.0.borrow_mut()` and panics.

```
thread '<unnamed>' panicked at tauri-runtime-wry-2.11.2/src/lib.rs:4086:19:
RefCell already borrowed
```

## What it does

`src-tauri/src/lib.rs` creates a window when the app receives
`RunEvent::WindowEvent { event: WindowEvent::Resumed | Suspended }` — i.e. on every
background/foreground transition. Window creation is dispatched inline on the main
thread and hits `borrow_mut` while the runtime still holds the borrow → crash.

## Reproduce

Requires Rust, Node/Bun, and the Tauri mobile toolchains (Xcode / Android SDK+NDK).

```bash
npm install        # or bun install
npm run tauri ios init      # and/or: npm run tauri android init
npm run tauri ios dev       # or: npm run tauri android dev
```

On device: **background the app (Home), then foreground it.** It crashes with the
`BorrowMutError` above (iOS: visible in Xcode console; Android: `adb logcat`, tag
`RustStdoutStderr`). Confirmed on both iOS and Android.

> Note: use the Home gesture to background — don't force-quit from the app switcher,
> as a cold relaunch doesn't fire the resume lifecycle event.

## RED vs GREEN

- **RED (buggy):** build against published Tauri — no `[patch.crates-io]` in
  `src-tauri/Cargo.toml`. Crashes on resume/suspend.
- **GREEN (fixed):** the `[patch.crates-io]` block in `src-tauri/Cargo.toml` points the
  Tauri crates at a local checkout that contains the fix (drop the `windows` borrow
  before dispatch). Rebuild → the window is created cleanly, no crash.

Toggle by commenting the trigger's expected outcome in the console:
`[MRE] window built — NO crash` (fixed) vs the `BorrowMutError` panic (buggy).
