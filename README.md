# Ghosted

Drifting double-tracking ghosts, from ADT to slapback: a VST3 and CLAP plugin

<img src="docs/ghosted.png" width="434" alt="Ghosted, a vintage walnut and cream bakelite radio with an amber dial full of ghosts, a brass Ghosted badge in a puffy font on the speaker grille, a brass Distance tuning knob, 2 and 4 ghost band keys, and Haunt, Drift and Spook knobs, and a chrome footswitch under an amber pilot lamp">

For guitar, vocals or anything else, on macOS and Windows. It adds two
or four delayed copies of the input that wander in time like extra
takes, from tight ADT out to Abbey Road-style slapback.

## How it works

1. The input is summed to mono and fed to four delay lines: a tight and
   a loose voice on each side. In 2-voice mode only the tight pair
   plays, hard left and right.
2. Each voice's delay time drifts with smoothed random motion, which
   also bends its pitch a little, like a second take that isn't quite
   in time.
3. A transient detector catches pick attacks and kicks each voice early
   or late for a moment, so the ghosts don't land exactly together.
4. Each voice has its own band-pass EQ, so the copies sound like
   different takes rather than the same one repeated.

Knob moves glide, and fast Distance changes dip the pitch like tape
instead of clicking.

## Controls

| Control | Range | What it does |
|---|---|---|
| Haunt | 0–100% | Wet mix. The dry signal turns down as the ghosts come up |
| Distance | 5–150 ms | Base delay. Under about 40 ms it thickens like double tracking; above 80 ms it's a slapback |
| Drift | 0–100% | How far (up to 32 ms) and how fast (0.1–0.9 Hz) the ghosts wander, and how far the loose pair strays from the tight pair |
| Spook | 0–100% | How far (up to 20 ms) and how long (15–165 ms) each pick attack pushes the ghosts early or late |
| Voices | 2 / 4 | 2: the tight pair, hard left and right. 4: adds a loose, later voice on each side |
| Stomp | On / Off | The footswitch. Off fades the ghosts out and leaves the dry signal untouched |

Drag a knob up or down (hold Shift for fine moves), scroll, or use the
arrow keys. Double-click a knob to reset it. Click the footswitch to
turn the ghosts on and off; the pilot lamp shows which.

## Tips

- For classic ADT, keep Distance under 30 ms and Haunt around 50%.
- For a '50s slapback on a vocal or a rockabilly guitar, set Voices to 2,
  Distance to 90–130 ms, and Drift low.
- Turn Spook down on strummed parts, where every strum is a transient
  and the ghosts never settle.

## Installing

Download the zip from
[Releases](https://github.com/starzonmyarmz/ghosted/releases) and follow
`INSTALL.txt` inside it.

## Building

```sh
cargo xtask bundle ghosted --release       # target/bundled/Ghosted.{vst3,clap}
scripts/install.sh                         # bundle, sign ad hoc and install on macOS as X.Y.Z-dev+<sha>
scripts/clap-validator.sh target/bundled/Ghosted.clap
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
```

Local installs call themselves `X.Y.Z-dev+<sha>` on the panel, so you
can tell which build the host loaded. Only releases change the version.

## Tests

`cargo test` runs the engine offline: a guitar DI rendered against
stored stereo golden renders (double-tracked and slapback), plus
synthetic tests for the controls, gliding and sample rates. After an
intended change to the sound, rerun with `BLESS=1` to rewrite the
renders, and listen to them before committing.

## Working on the UI

The editor is a webview showing `ui/index.html`, which is compiled into
the plugin. Without the plugin, the page runs against a mock, so you can
work on it in a browser:

```sh
cd ui && python3 -m http.server 8765    # open http://127.0.0.1:8765
```

Add `?version=1.2.0` to show a version other than `0.0.0`. To load
the page live in the plugin, set `GHOSTED_UI_URL` to that URL
before starting the host.

## CI and releases

Versions follow [SemVer](https://semver.org), and the commits since the
last release decide the bump:

| Commits since the last release | Bump |
|---|---|
| a `!` or `BREAKING CHANGE:` footer | major (minor before 1.0) |
| `feat` | minor |
| `fix`, `perf`, `revert` | patch |
| anything else | no release |

`scripts/release` shows the computed bump and the changelog, then
commits, tags `vX.Y.Z` and pushes. Pass `patch`, `minor`, `major` or an
exact version to override, and `--dry-run` to only look. It refuses to
release `feat`, `fix` or `perf` commits while `[Unreleased]` in
[CHANGELOG.md](CHANGELOG.md) is empty.

- `check.yml` runs on pushes to main and on PRs: `cargo fmt --check`
  and clippy, the tests and clap-validator on macOS, and clap-validator
  on the Windows build.
- `changelog.yml` runs on PRs and fails when a `feat`, `fix` or `perf`
  commit doesn't touch `CHANGELOG.md`. Add the `no-changelog` label or
  put `[skip changelog]` in the subject to opt out.
- `release.yml` runs on `vX.Y.Z` tags. The tag must match `version` in
  the root `Cargo.toml`. It tests, builds a universal macOS bundle and a
  Windows bundle, validates both, and publishes `ghosted-X.Y.Z.zip`
  with `INSTALL.txt` and `SHA256SUMS`, using the version's changelog
  section as the release notes.
- Nothing on GitHub takes screenshots. `scripts/release` retakes
  `docs/ghosted.png` through `scripts/pre-release.sh` before the release
  commit, so the shot shows the new version. Between releases, retake it
  with `scripts/screenshot.sh` and commit it with the UI change it shows.

## Built with

- [nice-plug](https://codeberg.org/RustAudio/nice-plug), pinned to
  [a fork](https://github.com/starzonmyarmz/nice-plug/tree/patina) that
  adds `ActivateContext::gui_context()`.
- [Jost](https://github.com/indestructible-type/Jost) by Owen Earl and
  [Modak](https://github.com/EkType/Modak) by Ek Type,
  under the SIL Open Font License (`ui/fonts`).
- Started from
  [plugin-template](https://github.com/starzonmyarmz/plugin-template).
