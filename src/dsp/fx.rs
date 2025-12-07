use crate::dsp::{fast_tanh, flush_denormals, time_to_coeff};
use crate::params::GtrParams;

/// Simple circular delay line for basic space/reverb-ish effect.
struct DelayLine {
    buf: Vec<f32>,
    idx: usize,
}

impl DelayLine {
    fn new(len: usize) -> Self {
        Self {
            buf: vec![0.0; len.max(1)],
            idx: 0,
        }
    }

    #[inline]
    fn process(&mut self, x: f32, feedback: f32) -> f32 {
        let y = self.buf[self.idx];
        self.buf[self.idx] = flush_denormals(x + y * feedback);
        self.idx += 1;
        if self.idx >= self.buf.len() {
            self.idx = 0;
        }
        y
    }
}

struct Limiter {
    sr: f32,
    env: f32,
    ceiling: f32,
    release_coeff: f32,
}

impl Limiter {
    fn new(sr: f32) -> Self {
        let mut l = Self {
            sr,
            env: 0.0,
            ceiling: 0.98,
            release_coeff: 0.0,
        };
        l.update_params(sr);
        l
    }

    fn update_params(&mut self, sr: f32) {
        self.sr = sr;
        let release_s = 0.05; // 50ms-ish
        self.release_coeff = time_to_coeff(release_s, self.sr);
    }

    #[inline]
    fn process(&mut self, l: f32, r: f32) -> (f32, f32) {
        let peak = l.abs().max(r.abs());
        if peak > self.env {
            self.env = peak;
        } else {
            self.env *= self.release_coeff;
        }

        // simple hard ceiling with smoothed env
        let mut gl = 1.0;
        if self.env > self.ceiling {
            gl = self.ceiling / self.env.max(1e-9);
        }

        // a touch of softening to avoid brickwall artifacts
        let l2 = gl * l;
        let r2 = gl * r;
        (flush_denormals(l2), flush_denormals(r2))
    }
}

/// Stereo FX block: handles "Space" (simple feedback delay/reverb),
/// "Width" (M/S widening), and safety limiter / output gain.
pub struct StereoFx {
    sr: f32,
    delay_l: DelayLine,
    delay_r: DelayLine,
    limiter: Limiter,
}

impl StereoFx {
    pub fn new(sr: f32) -> Self {
        // Use relatively short prime-ish delays
        let dl = (0.040 * sr as f32) as usize; // ~40ms
        let dr = (0.047 * sr as f32) as usize; // ~47ms

        Self {
            sr,
            delay_l: DelayLine::new(dl),
            delay_r: DelayLine::new(dr),
            limiter: Limiter::new(sr),
        }
    }

    pub fn reset(&mut self, sr: f32) {
        self.sr = sr;
        self.delay_l = DelayLine::new((0.040 * sr as f32) as usize);
        self.delay_r = DelayLine::new((0.047 * sr as f32) as usize);
        self.limiter.update_params(sr);
    }

    #[inline]
    pub fn process_frame(&mut self, dry_l: f32, dry_r: f32, p: &GtrParams) -> (f32, f32) {
        let space = p.space.value().clamp(0.0, 1.0);
        let width = p.width.value().clamp(0.0, 1.0);

        // Basic stereo feedback delay network
        let fb = 0.25 + 0.4 * space; // more feedback with higher space

        let dl = self.delay_l.process(dry_l, fb);
        let dr = self.delay_r.process(dry_r, fb);

        // crossfeed to make it a bit more spacious
        let wet_l = dl + 0.3 * dr;
        let wet_r = dr + 0.3 * dl;

        let mix = space; // 0..1
        let mut l = dry_l * (1.0 - mix) + wet_l * mix;
        let mut r = dry_r * (1.0 - mix) + wet_r * mix;

        // Width via M/S
        let mid = 0.5 * (l + r);
        let side = 0.5 * (l - r);
        let side_gain = 1.0 + width * 1.5; // up to ~+3.5dB side
        let new_side = side * side_gain;

        l = mid + new_side;
        r = mid - new_side;

        // Apply output gain then limiter if enabled
        let out_gain = p.output_linear();
        l *= out_gain;
        r *= out_gain;

        if p.limiter_on.value() {
            (l, r) = self.limiter.process(l, r);
        }

        // Final soft clipper as safety
        let l = fast_tanh(l);
        let r = fast_tanh(r);

        (flush_denormals(l), flush_denormals(r))
    }
}
