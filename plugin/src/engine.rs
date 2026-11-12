use std::f64::consts::PI;

/// Each voice's high-pass and low-pass corners, in Hz.
const VOICE_EQ: [(f64, f64); 4] = [(150.0, 3500.0), (260.0, 6500.0), (190.0, 4700.0), (100.0, 10000.0)];

const SEED: u64 = 0x6705_7ED0_6705_7ED0;

/// The most a gliding delay time moves per sample, in samples, for each of
/// base and depth. Together they keep a fast knob turn to at most an octave
/// of tape-style pitch dip, instead of reading the delay line backwards.
const MAX_SLEW: f64 = 0.25;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Voices {
    /// The tight pair, hard left and right.
    Two,
    /// A tight and a loose voice on each side.
    Four,
}

/// The front-panel controls. Each knob sets several of the engine's
/// [`Settings`].
#[derive(Clone, Copy, Debug)]
pub struct Controls {
    /// Wet mix, 0 to 1.
    pub haunt: f64,
    /// Base delay, 5 to 150 ms.
    pub distance_ms: f64,
    /// 0 to 1: how far and how fast the ghosts wander, and how far the
    /// loose pair strays from the tight pair.
    pub drift: f64,
    /// 0 to 1: how far and how long each pick attack pushes the ghosts early
    /// or late.
    pub spook: f64,
    pub voices: Voices,
    /// The footswitch. Off fades the ghosts out; the dry signal always
    /// passes.
    pub stomp: bool,
}

impl Default for Controls {
    fn default() -> Self {
        Self { haunt: 0.5, distance_ms: 20.0, drift: 0.5, spook: 0.5, voices: Voices::Four, stomp: true }
    }
}

impl Controls {
    pub fn settings(&self) -> Settings {
        Settings {
            wet: if self.stomp { self.haunt } else { 0.0 },
            base_ms: self.distance_ms,
            drift_ms: 32.0 * self.drift * self.drift,
            drift_hz: 0.1 * 9f64.powf(self.drift),
            looseness: self.drift,
            jitter_ms: 20.0 * self.spook * self.spook,
            jitter_dur_ms: 15.0 + 150.0 * self.spook,
            sensitivity: 1.0,
            voices: self.voices,
        }
    }
}

/// The engine's raw values, which [`Controls`] maps the knobs onto.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub wet: f64,
    pub base_ms: f64,
    pub drift_ms: f64,
    pub drift_hz: f64,
    /// Tight/Loose amount, 0 to 1.
    pub looseness: f64,
    pub jitter_ms: f64,
    pub jitter_dur_ms: f64,
    /// Transient sensitivity, 0.2 to 3 (20–300%).
    pub sensitivity: f64,
    pub voices: Voices,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            wet: 0.5,
            base_ms: 20.0,
            drift_ms: 8.0,
            drift_hz: 0.3,
            looseness: 0.5,
            jitter_ms: 5.0,
            jitter_dur_ms: 90.0,
            sensitivity: 1.0,
            voices: Voices::Four,
        }
    }
}

/// Zeroes values too small to hear, so decaying filters never reach
/// subnormals, which are slow on x86 and which hosts reject in output.
fn flush(x: f64) -> f64 {
    if x.abs() < 1e-30 {
        0.0
    } else {
        x
    }
}

/// xorshift64*, so renders repeat exactly.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> f64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        (x.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Targets that glide to their new value instead of jumping, so turning a
/// knob doesn't click.
#[derive(Clone, Copy, Default)]
struct Glide {
    base: f64,
    depth: f64,
    gl: f64,
    gr: f64,
}

impl Glide {
    fn toward(&mut self, target: &Glide, a: f64) {
        self.base += ((target.base - self.base) * a).clamp(-MAX_SLEW, MAX_SLEW);
        self.depth += ((target.depth - self.depth) * a).clamp(-MAX_SLEW, MAX_SLEW);
        self.gl += (target.gl - self.gl) * a;
        self.gr += (target.gr - self.gr) * a;
    }
}

struct Voice {
    buf: Vec<f64>,
    wpos: usize,
    ah: f64,
    al: f64,
    dphase: f64,
    dprev: f64,
    dnext: f64,
    jtgt: f64,
    jrem: f64,
    jcur: f64,
    xp: f64,
    hp: f64,
    lp: f64,
    target: Glide,
    now: Glide,
}

impl Voice {
    fn new(len: usize, sr: f64, (hpf, lpf): (f64, f64)) -> Self {
        Self {
            buf: vec![0.0; len],
            wpos: 0,
            ah: (-2.0 * PI * hpf / sr).exp(),
            al: (-2.0 * PI * lpf / sr).exp(),
            dphase: 0.0,
            dprev: 0.0,
            dnext: 0.0,
            jtgt: 0.0,
            jrem: 0.0,
            jcur: 0.0,
            xp: 0.0,
            hp: 0.0,
            lp: 0.0,
            target: Glide::default(),
            now: Glide::default(),
        }
    }

    fn reset(&mut self, rng: &mut Rng) {
        self.buf.fill(0.0);
        self.wpos = 0;
        self.dphase = rng.next();
        self.dprev = rng.next() * 2.0 - 1.0;
        self.dnext = rng.next() * 2.0 - 1.0;
        self.jtgt = 0.0;
        self.jrem = 0.0;
        self.jcur = 0.0;
        self.xp = 0.0;
        self.hp = 0.0;
        self.lp = 0.0;
    }
}

pub struct Engine {
    sample_rate: f32,
    sr: f64,
    voices: [Voice; 4],
    rng: Rng,
    fresh: bool,

    jsmooth: f64,
    env_att: f64,
    env_rel: f64,
    env_slow: f64,
    glide: f64,
    trighang_max: u32,

    dinc: f64,
    thrmul: f64,
    jit_samp: f64,
    jit_dur: f64,
    dry_target: f64,
    dry: f64,

    envf: f64,
    envs: f64,
    trighang: u32,
}

impl Engine {
    pub fn new(sample_rate: f32) -> Self {
        let sr = sample_rate as f64;
        // About one second, comfortably over the ~700 ms worst case.
        let len = sr.ceil() as usize;
        let coef = |seconds: f64| 1.0 - (-1.0 / (sr * seconds)).exp();
        let mut engine = Self {
            sample_rate,
            sr,
            voices: VOICE_EQ.map(|eq| Voice::new(len, sr, eq)),
            rng: Rng(SEED),
            fresh: true,
            jsmooth: coef(0.005),
            env_att: coef(0.0005),
            env_rel: coef(0.02),
            env_slow: coef(0.15),
            glide: coef(0.05),
            trighang_max: (sr * 0.04).floor() as u32,
            dinc: 0.0,
            thrmul: 0.0,
            jit_samp: 0.0,
            jit_dur: 1.0,
            dry_target: 0.0,
            dry: 0.0,
            envf: 0.0,
            envs: 0.0,
            trighang: 0,
        };
        engine.set(Settings::default());
        engine.reset();
        engine
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    /// Clears the delay lines and restarts the random drift from the same
    /// seed, so every render of the same input sounds the same.
    pub fn reset(&mut self) {
        self.rng = Rng(SEED);
        for v in &mut self.voices {
            v.reset(&mut self.rng);
        }
        self.envf = 0.0;
        self.envs = 0.0;
        self.trighang = 0;
        self.fresh = true;
    }

    pub fn set(&mut self, s: Settings) {
        let sr = self.sr;
        let tl = s.looseness;
        self.dry_target = 1.0 - s.wet;
        self.dinc = s.drift_hz / sr;
        self.thrmul = 1.0 + 1.5 / s.sensitivity;
        self.jit_samp = s.jitter_ms * sr / 1000.0;
        self.jit_dur = (s.jitter_dur_ms * sr / 1000.0).floor().max(1.0);

        // The loose pair keeps running when muted so switching back to four
        // doesn't replay stale audio.
        let pans = match s.voices {
            Voices::Four => [(0.0, 0.24), (0.2, 0.24), (0.8, 0.24), (1.0, 0.24)],
            Voices::Two => [(0.0, 0.34), (0.2, 0.0), (1.0, 0.34), (1.0, 0.0)],
        };
        let base = s.base_ms * sr / 1000.0;
        let drift = s.drift_ms * sr / 1000.0;
        let loose_mul = 1.0 + tl * 1.2;
        let bases = [base * 0.65, base * 1.55 * loose_mul, base * 0.80, base * 1.90 * loose_mul];
        let loose_depth = drift * (1.3 + tl * 0.5);
        let depths = [drift * 0.5, loose_depth, drift * 0.5, loose_depth];

        for (i, v) in self.voices.iter_mut().enumerate() {
            let (pan, gain) = pans[i];
            v.target = Glide {
                base: bases[i],
                depth: depths[i],
                gl: (pan * PI / 2.0).cos() * gain * s.wet,
                gr: (pan * PI / 2.0).sin() * gain * s.wet,
            };
            if self.fresh {
                v.now = v.target;
            }
        }
        if self.fresh {
            self.dry = self.dry_target;
            self.fresh = false;
        }
    }

    pub fn process(&mut self, left: &mut [f32], right: &mut [f32]) {
        let maxd = self.voices[0].buf.len();
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let (inl, inr) = (*l as f64, *r as f64);
            let mono = (inl + inr) * 0.5;

            let env = mono.abs();
            let att = if env > self.envf { self.env_att } else { self.env_rel };
            self.envf = flush(self.envf + (env - self.envf) * att);
            self.envs = flush(self.envs + (env - self.envs) * self.env_slow);

            let mut trig = false;
            if self.trighang > 0 {
                self.trighang -= 1;
            } else if self.envf > self.envs * self.thrmul + 0.002 && self.envf > 0.008 {
                trig = true;
                self.trighang = self.trighang_max;
            }

            self.dry += (self.dry_target - self.dry) * self.glide;
            let (mut wl, mut wr) = (0.0, 0.0);
            for v in &mut self.voices {
                v.now.toward(&v.target, self.glide);

                v.dphase += self.dinc;
                if v.dphase >= 1.0 {
                    v.dphase -= 1.0;
                    v.dprev = v.dnext;
                    v.dnext = self.rng.next() * 2.0 - 1.0;
                }
                let drift = v.dprev + (v.dnext - v.dprev) * (0.5 - 0.5 * (PI * v.dphase).cos());

                if trig {
                    v.jtgt = (self.rng.next() * 2.0 - 1.0) * self.jit_samp;
                    v.jrem = self.jit_dur;
                }
                let jt = if v.jrem > 0.0 { v.jtgt * v.jrem / self.jit_dur } else { 0.0 };
                if v.jrem > 0.0 {
                    v.jrem -= 1.0;
                }
                v.jcur = flush(v.jcur + (jt - v.jcur) * self.jsmooth);

                let d = (v.now.base + drift * v.now.depth + v.jcur).min(maxd as f64 - 2.0).max(1.0);
                let mut rpos = v.wpos as f64 - d;
                if rpos < 0.0 {
                    rpos += maxd as f64;
                }
                let i0 = rpos.floor() as usize;
                let i1 = if i0 + 1 >= maxd { 0 } else { i0 + 1 };
                let f = rpos - i0 as f64;
                let x = v.buf[i0] * (1.0 - f) + v.buf[i1] * f;

                v.buf[v.wpos] = mono;
                v.wpos += 1;
                if v.wpos >= maxd {
                    v.wpos = 0;
                }

                v.hp = flush(v.ah * (v.hp + x - v.xp));
                v.xp = x;
                v.lp = flush(v.al * v.lp + (1.0 - v.al) * v.hp);
                wl += v.lp * v.now.gl;
                wr += v.lp * v.now.gr;
            }

            *l = flush(inl * self.dry + wl) as f32;
            *r = flush(inr * self.dry + wr) as f32;
        }
    }
}
