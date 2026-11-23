# Design

How Ghosted looks and feels. When this file and your instincts disagree, this file wins. Change it on purpose, in the same commit as the code.

## The look in one line

A vintage walnut and cream bakelite radio whose amber dial is full of ghosts.

## Principles

Every plugin gets its own look, but they share an approach.

- **Skeuomorphic.** The editor is a physical object (Patina an OP-1-style synth, Halfwit a stompbox, Ghosted a vintage radio), with real materials like powder coat, bakelite, walnut, brass and grille cloth, built in CSS gradients and shadows rather than images.
- **One theme.** Pick the object from the plugin's vibe and name, and let it run through everything: the wordmark, the art and the control labels. Names should be playful and capture the vibe.
- **Restrained.** One knob style and colour family per plugin, and few controls. Fold many settings into a handful of knobs with evocative names rather than exposing every one.
- **Colour and art mean something.** If two waves or knobs differ in colour, it's because they differ in function; otherwise make them the same. Art can follow parameters (Ghosted's ghosts fade with Haunt, the dial needle follows Distance), but don't fake audio metering.
- **Plain labels.** Words a guitarist understands, not DSP terms or fieldset jargon. Numbers go in a tooltip readout.
- **Type with character.** Embed a display face for the wordmark (Futura-like Jost, pillowy Modak), never a system or "commercial" font, and centre it optically on its badge.
- **Drawn, not vector-perfect.** Vary repeated elements (size, tilt, expression), and give outlines weight that swells on the shadow side, like inked comic art.
- **A footswitch on effects.** Pedals and other effects get an on/off toggle (`stomp`, on by default). Off fades the effect out and passes the dry signal; a lamp or LED shows the state, and the art reacts (Ghosted's dial dims and its ghosts leave).
- **The version on the panel.** Print it small and quiet as part of the object, like a model number on a pedal's base plate or a serial plate on a radio's back (`v1.2.0`), so anyone can tell which build they're running. Local installs show `X.Y.Z-dev+<sha>`.
- **Physical details behave physically.** Shadows stay put while a knob turns, pressed keys sink, lit things glow.

## This plugin

Set by the `design-kickoff` skill.

- **Object:** a tabletop radio: walnut cabinet on feet, cream bakelite face, grille cloth speaker, amber tuning dial.
- **Palette:** walnut, cream bakelite, brass, amber dial glow, chrome for the footswitch.
- **Wordmark font:** Modak, on a brass badge on the speaker grille. Labels are in Jost.
- **Controls:** a Distance tuning knob (its needle moves across the dial), 2 and 4 ghost band keys for Voices, Haunt, Drift and Spook knobs, and a chrome footswitch under an amber pilot lamp. The ghosts on the dial fade with Haunt. Off dims the dial, darkens the pilot lamp and hides the dial ghosts.
- **Version placement:** small, at the bottom centre of the cream face, in the dial's ink.

## Tokens

Every color, size and space comes from a token. If nothing fits, add a token here before using a new value. They live on `:root` in `ui/index.html`; most come in `-hi`/`-lo` pairs for highlights and shadows.

| Token | Value | Used for |
|---|---|---|
| `--board` | `#1c1611` | the window behind the radio |
| `--walnut` | `#6a3c1f` | the cabinet |
| `--bakelite` | `#efe3c6` | the cream face |
| `--ink`, `--ink-soft` | `#4a3220` | dial marks, labels and the version |
| `--knob` | `#5a321b` | the knobs' ribbed skirts |
| `--brass` | `#d1a54e` | knob caps, the badge, and the dial and lamp bezels |
| `--glow`, `--glow-hot`, `--glow-edge` | `#ffd98a` | the lit dial |
| `--cloth` | `#b98f5a` | the grille cloth |
| `--ghost` | `#fffdf4` | the ghosts |
| `--chrome` | `#c4c6c8` | the footswitch |
| `--lamp-off` | `#6b3a12` | the pilot lamp when off |
| `--label` | Jost | control labels |

## Components

## Patterns

## Adding something new

1. Look for an existing component or pattern that already does it.
2. Build it from tokens only.
3. Check Reduce Motion if it moves.
4. Retake the screenshot if it shows in one.
5. Record it above.

## Known drift

Existing debt to clean up, not to copy.

- The off pilot lamp's highlight is a raw `#c98a4a`, not a token.
