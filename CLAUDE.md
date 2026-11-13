# Ghosted

Read `README.md` first for what the plugin does, its controls, and how to
build and run it. This file has the maintainer details it leaves out.

## Layout

- `plugin/src/engine.rs`: all DSP, with no plugin framework types, so tests
  drive it directly. `Settings` holds the engine's raw values;
  `Controls` is the front panel (four knobs, Voices and the footswitch),
  and `Controls::settings()` maps it onto them.
- `plugin/src/params.rs`: host parameters, mapped to `Controls`.
- `plugin/src/editor.rs` and `plugin/src/bridge.rs`: nothing specific to
  Ghosted. The page's size and HTML come from `PAGE` in `lib.rs`, and the
  bridge works with any `#[derive(Params)]` struct.
- `ui/index.html`: the whole editor, compiled into the plugin, with a mock
  bridge for running it in a browser.

## Random drift

Drift and jitter come from a fixed-seed xorshift64* that restarts on
reset, so renders repeat exactly and the golden tests can compare them.
Changing the order of random draws (per voice: phase, prev, next on
reset; next drift target, then jitter target, while processing) changes
the sound, so it needs new golden renders.

## Adding a parameter

It has to go in six places:

1. `Controls` in `engine.rs` and its `Default`, plus its mapping in
   `settings()`. If it needs a new engine value, `Settings` too.
2. `GhostedParams` in `params.rs`, and its `settings()` mapping.
3. The mock's `specs` in `ui/index.html`. The plugin's bridge picks new
   parameters up automatically, but the browser mock doesn't.
4. A `data-knob` (or `data-keys`, `data-stomp-switch`) element in the
   markup.
5. `decorate()` in `ui/index.html`, if the dial art should follow it.
6. The controls tables in `README.md`.

## Sound changes and golden renders

`plugin/tests/golden` holds stereo renders of `plugin/tests/di/riff.wav`
at the defaults and as a slapback. After an intended change to the sound,
rerun with `BLESS=1 cargo test` to rewrite them. Never commit rewritten
renders until the user has listened to them, and say in the commit what
changed in the sound.

## UI

- The window size is set in two places that must change together:
  `PAGE.size` in `lib.rs`, and `.radio`'s `inline-size` plus the face's
  content height in `ui/index.html`. The cabinet's feet hang 9 px below it.
- The footswitch's on/off state is `data-stomp` on `.radio`: off dims
  the dial, darkens the pilot lamp and hides the dial ghosts.
- Distance's dial marks use the host's skew: normalized =
  sqrt((ms − 5) / 145). Change `distanceAt` if the range or skew changes.
- The ghosts are stamped from `<template id="ghost">`, not `<use>`, so
  page CSS reaches their eyes and mouths.
- The version, from the root `Cargo.toml`, is printed small at the
  bottom centre of the cream face, in the dial's ink.
- The local preview server (`python3 -m http.server`) gets cached by the
  browser. Add a query string like `?v=2` after each edit.
- After a visible UI change, retake `docs/ghosted.jpg`: capture the full
  window at device scale 2 with reduced motion, save it as a JPEG 852 px
  tall, and set the README `<img>` width to half the JPEG's width.

## Releases

`release.yml` runs on `vX.Y.Z` tags, which must match `version` in the
root `Cargo.toml`. To release: bump `version`, commit, then
`git tag vX.Y.Z && git push origin main vX.Y.Z`.

## nice-plug fork

The plugin is pinned to the `patina` branch of a nice-plug fork, which
adds `ActivateContext::gui_context()`. To upgrade, rebase that branch onto
upstream, push it, and update the `rev` in both `plugin/Cargo.toml` and
`xtask/Cargo.toml`.
