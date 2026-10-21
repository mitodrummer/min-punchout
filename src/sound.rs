//! Chiptune audio. Two pulse channels, a triangle bass and a noise drum,
//! the NES recipe, rendered to PCM at startup and played through the OS
//! mixer with rodio. Nothing on disk, nothing to install.
//!
//! If no audio device opens (a sandbox, CI, a headless box) `Audio::new`
//! returns None and the game runs silent.

use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};

const RATE: u32 = 44_100;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sfx {
    /// The "press now" blip when the opponent turns red.
    Cue,
    /// A lighter blip for a jab cue, where the answer is block.
    CueJab,
    /// Your jab lands.
    Jab,
    /// Your hook or uppercut lands.
    Heavy,
    /// A double-damage counter lands.
    Counter,
    /// A punch is blocked, either way.
    Block,
    /// A punch whiffs through a dodge.
    Whoosh,
    /// You get hit.
    Hurt,
    /// Someone is stunned.
    Stun,
    /// A signature move is coming.
    Alarm,
    /// Round bell.
    Bell,
    Win,
    Lose,
    /// Card cursor moved.
    Menu,
    /// Card choice confirmed.
    Confirm,
}

const ALL: [Sfx; 16] = [
    Sfx::Cue,
    Sfx::CueJab,
    Sfx::Jab,
    Sfx::Heavy,
    Sfx::Counter,
    Sfx::Block,
    Sfx::Whoosh,
    Sfx::Hurt,
    Sfx::Stun,
    Sfx::Alarm,
    Sfx::Bell,
    Sfx::Win,
    Sfx::Lose,
    Sfx::Menu,
    Sfx::Confirm,
    Sfx::Cue,
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Music {
    Card,
    Fight,
    Off,
}

pub struct Audio {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    music: Option<Sink>,
    playing: Music,
    sfx: Vec<(Sfx, Vec<f32>)>,
    card_loop: Vec<f32>,
    fight_loop: Vec<f32>,
    pub music_on: bool,
}

impl Audio {
    pub fn new(music_on: bool) -> Option<Audio> {
        let (stream, handle) = OutputStream::try_default().ok()?;
        let sfx = ALL.iter().map(|s| (*s, render_sfx(*s))).collect();
        Some(Audio {
            _stream: stream,
            handle,
            music: None,
            playing: Music::Off,
            sfx,
            card_loop: render_song(&CARD_SONG),
            fight_loop: render_song(&FIGHT_SONG),
            music_on,
        })
    }

    pub fn play(&self, s: Sfx) {
        if let Some((_, data)) = self.sfx.iter().find(|(k, _)| *k == s) {
            let src = SamplesBuffer::new(1, RATE, data.clone());
            let _ = self.handle.play_raw(src.convert_samples());
        }
    }

    pub fn music(&mut self, m: Music) {
        if m == self.playing {
            return;
        }
        self.playing = m;
        // Dropping the old sink stops it.
        self.music = None;
        if !self.music_on || m == Music::Off {
            return;
        }
        let data = match m {
            Music::Card => self.card_loop.clone(),
            Music::Fight => self.fight_loop.clone(),
            Music::Off => return,
        };
        if let Ok(sink) = Sink::try_new(&self.handle) {
            sink.set_volume(0.55);
            sink.append(SamplesBuffer::new(1, RATE, data).repeat_infinite());
            self.music = Some(sink);
        }
    }
}

// ------------------------------------------------------------------ synth


fn freq(semitones_from_a4: i32) -> f32 {
    440.0 * 2f32.powf(semitones_from_a4 as f32 / 12.0)
}

/// Square wave with a duty cycle, the NES pulse channel.
fn pulse(phase: f32, duty: f32) -> f32 {
    if phase.fract() < duty { 1.0 } else { -1.0 }
}

fn triangle(phase: f32) -> f32 {
    4.0 * (phase.fract() - 0.5).abs() - 1.0
}

/// Linear feedback shift register noise, like the NES.
struct Noise(u16);
impl Noise {
    fn next(&mut self) -> f32 {
        let bit = (self.0 ^ (self.0 >> 1)) & 1;
        self.0 = (self.0 >> 1) | (bit << 14);
        if self.0 & 1 == 1 { 1.0 } else { -1.0 }
    }
}

/// One voice playing one note: a tone that sweeps from f0 to f1 over `len`
/// seconds with an attack/decay envelope.
fn tone(out: &mut Vec<f32>, start: f32, len: f32, f0: f32, f1: f32, duty: f32, gain: f32, tri: bool) {
    let s0 = (start * RATE as f32) as usize;
    let n = (len * RATE as f32) as usize;
    if out.len() < s0 + n {
        out.resize(s0 + n, 0.0);
    }
    let mut phase = 0.0f32;
    for i in 0..n {
        let t = i as f32 / n as f32;
        let f = f0 + (f1 - f0) * t;
        phase += f / RATE as f32;
        let env = if t < 0.05 { t / 0.05 } else { 1.0 - (t - 0.05) / 0.95 };
        let v = if tri { triangle(phase) } else { pulse(phase, duty) };
        out[s0 + i] += v * env * gain;
    }
}

fn noise(out: &mut Vec<f32>, start: f32, len: f32, gain: f32, decay: f32) {
    let s0 = (start * RATE as f32) as usize;
    let n = (len * RATE as f32) as usize;
    if out.len() < s0 + n {
        out.resize(s0 + n, 0.0);
    }
    let mut lfsr = Noise(0x1ACE);
    let mut hold = 0.0;
    for i in 0..n {
        // Downsampled noise reads grittier, more like the chip.
        if i % 3 == 0 {
            hold = lfsr.next();
        }
        let t = i as f32 / n as f32;
        out[s0 + i] += hold * (1.0 - t).powf(decay) * gain;
    }
}

fn render_sfx(s: Sfx) -> Vec<f32> {
    let mut o = Vec::new();
    match s {
        Sfx::Cue => {
            tone(&mut o, 0.0, 0.06, 1400.0, 1400.0, 0.5, 0.35, false);
            tone(&mut o, 0.07, 0.06, 1400.0, 1400.0, 0.5, 0.35, false);
        }
        Sfx::CueJab => tone(&mut o, 0.0, 0.05, 1000.0, 1000.0, 0.25, 0.3, false),
        Sfx::Jab => {
            noise(&mut o, 0.0, 0.07, 0.4, 2.0);
            tone(&mut o, 0.0, 0.06, 500.0, 200.0, 0.5, 0.3, false);
        }
        Sfx::Heavy => {
            noise(&mut o, 0.0, 0.14, 0.5, 1.5);
            tone(&mut o, 0.0, 0.16, 260.0, 50.0, 0.5, 0.45, false);
        }
        Sfx::Counter => {
            noise(&mut o, 0.0, 0.14, 0.5, 1.5);
            tone(&mut o, 0.0, 0.16, 260.0, 50.0, 0.5, 0.45, false);
            tone(&mut o, 0.08, 0.08, freq(12), freq(12), 0.5, 0.3, false);
            tone(&mut o, 0.16, 0.12, freq(19), freq(19), 0.5, 0.3, false);
        }
        Sfx::Block => tone(&mut o, 0.0, 0.04, 900.0, 600.0, 0.125, 0.35, false),
        Sfx::Whoosh => noise(&mut o, 0.0, 0.12, 0.18, 0.7),
        Sfx::Hurt => {
            tone(&mut o, 0.0, 0.18, 300.0, 90.0, 0.5, 0.4, true);
            noise(&mut o, 0.0, 0.08, 0.3, 2.0);
        }
        Sfx::Stun => {
            for k in 0..6 {
                let f = if k % 2 == 0 { 700.0 } else { 520.0 };
                tone(&mut o, k as f32 * 0.07, 0.06, f, f, 0.25, 0.25, false);
            }
        }
        Sfx::Alarm => {
            for k in 0..4 {
                let f = if k % 2 == 0 { freq(7) } else { freq(1) };
                tone(&mut o, k as f32 * 0.1, 0.09, f, f, 0.5, 0.3, false);
            }
        }
        Sfx::Bell => {
            tone(&mut o, 0.0, 0.7, 1760.0, 1760.0, 0.5, 0.3, true);
            tone(&mut o, 0.0, 0.4, 2637.0, 2637.0, 0.125, 0.12, false);
            tone(&mut o, 0.35, 0.7, 1760.0, 1760.0, 0.5, 0.3, true);
        }
        Sfx::Win => {
            for (k, n) in [3, 7, 10, 15].iter().enumerate() {
                tone(&mut o, k as f32 * 0.12, 0.11, freq(*n), freq(*n), 0.5, 0.3, false);
            }
            tone(&mut o, 0.48, 0.5, freq(15), freq(15), 0.5, 0.3, false);
            tone(&mut o, 0.48, 0.5, freq(3), freq(3), 0.25, 0.2, false);
        }
        Sfx::Lose => {
            for (k, n) in [0, -4, -7].iter().enumerate() {
                tone(&mut o, k as f32 * 0.22, 0.2, freq(*n), freq(*n - 1), 0.5, 0.3, false);
            }
            tone(&mut o, 0.66, 0.6, freq(-12), freq(-13), 0.5, 0.3, true);
        }
        Sfx::Menu => tone(&mut o, 0.0, 0.04, 800.0, 800.0, 0.25, 0.3, false),
        Sfx::Confirm => {
            tone(&mut o, 0.0, 0.06, freq(3), freq(3), 0.5, 0.3, false);
            tone(&mut o, 0.07, 0.1, freq(10), freq(10), 0.5, 0.3, false);
        }
    }
    o
}

// ------------------------------------------------------------- sequencer

const R: i32 = i32::MIN; // rest

/// A note as (semitones from A4, length in sixteenths).
type Track = &'static [(i32, u32)];

struct Song {
    bpm: f32,
    /// Lead, pulse at 50% duty.
    lead: Track,
    /// Harmony, pulse at 25% duty.
    harm: Track,
    /// Bass, triangle.
    bass: Track,
    /// Drums, one char per sixteenth: K kick, S snare, h hat, . rest.
    drums: &'static str,
}

// Pitches, semitones from A4.
const D2: i32 = -31;
const A2: i32 = -24;
const D3: i32 = -19;
const CS4: i32 = -8;
const CS5: i32 = 4;
const C3: i32 = -21;
const F3: i32 = -16;
const G3: i32 = -14;
const A3: i32 = -12;
const B3: i32 = -10;
const C4: i32 = -9;
const D4: i32 = -7;
const E4: i32 = -5;
const F4: i32 = -4;
const G4: i32 = -2;
const A4: i32 = 0;
const B4: i32 = 2;
const C5: i32 = 3;
const D5: i32 = 5;
const E5: i32 = 7;
const F5: i32 = 8;
const G5: i32 = 10;
const A5: i32 = 12;
const E2: i32 = -29;
const F2: i32 = -28;
const G2: i32 = -26;
const C6: i32 = 15;
const BB2: i32 = -23;
const BB3: i32 = -11;
const BB4: i32 = 1;
const BB5: i32 = 13;

/// Fight loop: A minor, fast, the melody pushing up and falling back.
const FIGHT_SONG: Song = Song {
    bpm: 165.0,
    lead: &[
        (A4, 2), (A4, 2), (C5, 2), (A4, 2), (E5, 4), (D5, 2), (C5, 2),
        (B4, 2), (B4, 2), (D5, 2), (B4, 2), (E5, 4), (G4, 4),
        (A4, 2), (A4, 2), (C5, 2), (A4, 2), (E5, 4), (G5, 2), (E5, 2),
        (D5, 2), (C5, 2), (B4, 2), (C5, 2), (A4, 8),
        (A4, 2), (A4, 2), (C5, 2), (A4, 2), (E5, 4), (D5, 2), (C5, 2),
        (B4, 2), (B4, 2), (D5, 2), (B4, 2), (E5, 4), (G4, 4),
        (A5, 2), (G5, 2), (E5, 2), (D5, 2), (C5, 2), (D5, 2), (E5, 4),
        (D5, 2), (C5, 2), (B4, 2), (G4, 2), (A4, 4), (R, 4),
    ],
    harm: &[
        (R, 2), (C4, 2), (R, 2), (C4, 2), (R, 2), (E4, 2), (R, 2), (E4, 2),
        (R, 2), (B3, 2), (R, 2), (B3, 2), (R, 2), (D4, 2), (R, 2), (D4, 2),
        (R, 2), (A3, 2), (R, 2), (A3, 2), (R, 2), (C4, 2), (R, 2), (C4, 2),
        (R, 2), (B3, 2), (R, 2), (B3, 2), (R, 2), (E4, 2), (R, 2), (E4, 2),
        (R, 2), (C4, 2), (R, 2), (C4, 2), (R, 2), (E4, 2), (R, 2), (E4, 2),
        (R, 2), (B3, 2), (R, 2), (B3, 2), (R, 2), (D4, 2), (R, 2), (D4, 2),
        (R, 2), (A3, 2), (R, 2), (A3, 2), (R, 2), (C4, 2), (R, 2), (C4, 2),
        (R, 2), (B3, 2), (R, 2), (B3, 2), (E4, 4), (R, 4),
    ],
    bass: &[
        (A2, 2), (A2, 2), (A2, 2), (A2, 2), (A2, 2), (A2, 2), (A3, 2), (A2, 2),
        (G2, 2), (G2, 2), (G2, 2), (G2, 2), (G2, 2), (G2, 2), (G3, 2), (G2, 2),
        (F2, 2), (F2, 2), (F2, 2), (F2, 2), (F2, 2), (F2, 2), (F3, 2), (F2, 2),
        (E2, 2), (E2, 2), (E2, 2), (E2, 2), (E2, 2), (E2, 2), (G2, 2), (G2, 2),
        (A2, 2), (A2, 2), (A2, 2), (A2, 2), (A2, 2), (A2, 2), (A3, 2), (A2, 2),
        (G2, 2), (G2, 2), (G2, 2), (G2, 2), (G2, 2), (G2, 2), (G3, 2), (G2, 2),
        (F2, 2), (F2, 2), (F2, 2), (F2, 2), (F2, 2), (F2, 2), (F3, 2), (F2, 2),
        (E2, 2), (E2, 2), (E2, 2), (E2, 2), (E2, 4), (R, 4),
    ],
    drums: "K.h.S.h.K.hKS.hhK.h.S.h.K.hKS.hhK.h.S.h.K.hKS.hhK.h.S.h.K.hKSSSS\
            K.h.S.h.K.hKS.hhK.h.S.h.K.hKS.hhK.h.S.h.K.hKS.hhK.h.S.h.K.hKS...",
};

/// Card theme: the select-screen groove, in D minor. The same pumping
/// bass and staccato lead, over Dm, Bb, C and an A major that pulls it
/// back down. Dark, not sad.
const CARD_SONG: Song = Song {
    bpm: 150.0,
    lead: &[
        (D5, 1), (R, 1), (F5, 1), (R, 1), (A5, 2), (F5, 2), (D5, 2), (E5, 2), (F5, 4),
        (D5, 1), (R, 1), (F5, 1), (R, 1), (BB5, 2), (A5, 2), (F5, 2), (D5, 2), (F5, 4),
        (E5, 2), (G5, 2), (C6, 2), (G5, 2), (E5, 2), (D5, 2), (C5, 4),
        (CS5, 2), (E5, 2), (A5, 2), (E5, 2), (CS5, 2), (D5, 2), (E5, 2), (R, 2),
        (D5, 1), (R, 1), (F5, 1), (R, 1), (A5, 2), (F5, 2), (D5, 2), (E5, 2), (F5, 4),
        (D5, 1), (R, 1), (F5, 1), (R, 1), (BB5, 2), (A5, 2), (F5, 2), (D5, 2), (F5, 4),
        (G5, 2), (A5, 2), (BB5, 2), (A5, 2), (G5, 2), (F5, 2), (E5, 4),
        (A5, 2), (G5, 2), (F5, 2), (E5, 2), (D5, 2), (CS5, 2), (D5, 2), (R, 2),
    ],
    harm: &[
        (D4, 2), (F4, 2), (A4, 2), (F4, 2), (D4, 2), (F4, 2), (A4, 2), (F4, 2),
        (D4, 2), (F4, 2), (BB4, 2), (F4, 2), (D4, 2), (F4, 2), (BB4, 2), (F4, 2),
        (E4, 2), (G4, 2), (C5, 2), (G4, 2), (E4, 2), (G4, 2), (C5, 2), (G4, 2),
        (CS4, 2), (E4, 2), (A4, 2), (E4, 2), (CS4, 2), (E4, 2), (A4, 2), (E4, 2),
        (D4, 2), (F4, 2), (A4, 2), (F4, 2), (D4, 2), (F4, 2), (A4, 2), (F4, 2),
        (D4, 2), (F4, 2), (BB4, 2), (F4, 2), (D4, 2), (F4, 2), (BB4, 2), (F4, 2),
        (E4, 2), (G4, 2), (C5, 2), (G4, 2), (E4, 2), (G4, 2), (C5, 2), (G4, 2),
        (CS4, 2), (E4, 2), (A4, 2), (E4, 2), (CS4, 2), (E4, 2), (A4, 2), (E4, 2),
    ],
    bass: &[
        (D2, 2), (D3, 1), (R, 1), (D2, 2), (D3, 2), (D2, 2), (D3, 1), (R, 1), (A2, 2), (D3, 2),
        (BB2, 2), (BB3, 1), (R, 1), (BB2, 2), (BB3, 2), (BB2, 2), (BB3, 1), (R, 1), (F2, 2), (BB3, 2),
        (C3, 2), (C4, 1), (R, 1), (C3, 2), (C4, 2), (C3, 2), (C4, 1), (R, 1), (G2, 2), (C4, 2),
        (A2, 2), (A3, 1), (R, 1), (A2, 2), (A3, 2), (A2, 2), (A3, 1), (R, 1), (E2, 2), (A3, 2),
        (D2, 2), (D3, 1), (R, 1), (D2, 2), (D3, 2), (D2, 2), (D3, 1), (R, 1), (A2, 2), (D3, 2),
        (BB2, 2), (BB3, 1), (R, 1), (BB2, 2), (BB3, 2), (BB2, 2), (BB3, 1), (R, 1), (F2, 2), (BB3, 2),
        (C3, 2), (C4, 1), (R, 1), (C3, 2), (C4, 2), (C3, 2), (C4, 1), (R, 1), (G2, 2), (C4, 2),
        (A2, 2), (A3, 1), (R, 1), (A2, 2), (A3, 2), (A2, 2), (A3, 1), (R, 1), (E2, 2), (A3, 2),
    ],
    drums: "KhhhShhhKhhhShhhKhhhShhhKhhhShhhKhhhShhhKhhhShhhKhhhShhhKhhhSSSS\
            KhhhShhhKhhhShhhKhhhShhhKhhhShhhKhhhShhhKhhhShhhKhhhShhhKhKhS.SS",
};

fn render_song(song: &Song) -> Vec<f32> {
    let sixteenth = 60.0 / song.bpm / 4.0;
    let total: u32 = song.lead.iter().map(|(_, l)| l).sum();
    let len = (total as f32 * sixteenth * RATE as f32) as usize;
    let mut out = vec![0.0f32; len];
    let mut lay = |track: Track, duty: f32, gain: f32, tri: bool| {
        let mut t = 0.0;
        for (n, l) in track {
            let d = *l as f32 * sixteenth;
            if *n != R {
                let f = freq(*n);
                // Leave a hair of silence between notes so repeats articulate.
                tone(&mut out, t, d * 0.9, f, f, duty, gain, tri);
            }
            t += d;
        }
    };
    lay(song.lead, 0.5, 0.16, false);
    lay(song.harm, 0.25, 0.1, false);
    lay(song.bass, 0.5, 0.2, true);
    for (i, ch) in song.drums.chars().filter(|c| !c.is_whitespace()).enumerate() {
        let t = i as f32 * sixteenth;
        match ch {
            'K' => {
                noise(&mut out, t, 0.05, 0.3, 2.0);
                tone(&mut out, t, 0.08, 150.0, 40.0, 0.5, 0.35, true);
            }
            'S' => noise(&mut out, t, 0.1, 0.25, 1.5),
            'h' => noise(&mut out, t, 0.025, 0.12, 1.0),
            _ => {}
        }
    }
    out.truncate(len);
    for v in &mut out {
        *v = v.clamp(-1.0, 1.0);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn songs_render_and_tracks_line_up() {
        for song in [&FIGHT_SONG, &CARD_SONG] {
            let lead: u32 = song.lead.iter().map(|(_, l)| l).sum();
            let harm: u32 = song.harm.iter().map(|(_, l)| l).sum();
            let bass: u32 = song.bass.iter().map(|(_, l)| l).sum();
            let drums = song.drums.chars().filter(|c| !c.is_whitespace()).count() as u32;
            assert_eq!(lead, harm, "harmony length");
            assert_eq!(lead, bass, "bass length");
            assert_eq!(lead, drums, "drum pattern length");
            let pcm = render_song(song);
            assert!(pcm.len() > RATE as usize);
            assert!(pcm.iter().any(|v| v.abs() > 0.1));
        }
    }

    #[test]
    fn every_effect_makes_a_sound() {
        for s in ALL {
            let pcm = render_sfx(s);
            assert!(!pcm.is_empty(), "{s:?}");
            assert!(pcm.iter().any(|v| v.abs() > 0.05), "{s:?}");
            assert!(pcm.iter().all(|v| v.abs() <= 1.5), "{s:?} clips hard");
        }
    }
}
