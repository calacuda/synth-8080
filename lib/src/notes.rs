use crate::Float;
use serde::{Deserialize, Serialize};
use std::convert::Into;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Deserialize, Serialize, Copy, PartialEq, Eq)]
pub enum Note {
    #[serde(alias = "c0", alias = "C0")]
    C0,
    #[serde(alias = "c#0", alias = "C#0")]
    Cs0,
    #[serde(alias = "db0", alias = "Db0")]
    Db0,
    #[serde(alias = "d0", alias = "D0")]
    D0,
    #[serde(alias = "d#0", alias = "D#0")]
    Ds0,
    #[serde(alias = "eb0", alias = "Eb0")]
    Eb0,
    #[serde(alias = "e0", alias = "E0")]
    E0,
    #[serde(alias = "f0", alias = "F0")]
    F0,
    #[serde(alias = "f#0", alias = "F#0")]
    Fs0,
    #[serde(alias = "gb0", alias = "Gb0")]
    Gb0,
    #[serde(alias = "g0", alias = "G0")]
    G0,
    #[serde(alias = "g#0", alias = "G#0")]
    Gs0,
    #[serde(alias = "ab0", alias = "Ab0")]
    Ab0,
    #[serde(alias = "a0", alias = "A0")]
    A0,
    #[serde(alias = "a#0", alias = "A#0")]
    As0,
    #[serde(alias = "bb0", alias = "Bb0")]
    Bb0,
    #[serde(alias = "b0", alias = "B0")]
    B0,
    #[serde(alias = "c1", alias = "C1")]
    C1,
    #[serde(alias = "c#1", alias = "C#1")]
    Cs1,
    #[serde(alias = "db1", alias = "Db1")]
    Db1,
    #[serde(alias = "d1", alias = "D1")]
    D1,
    #[serde(alias = "d#1", alias = "D#1")]
    Ds1,
    #[serde(alias = "eb1", alias = "Eb1")]
    Eb1,
    #[serde(alias = "e1", alias = "E1")]
    E1,
    #[serde(alias = "f1", alias = "F1")]
    F1,
    #[serde(alias = "f#1", alias = "F#1")]
    Fs1,
    #[serde(alias = "gb1", alias = "Gb1")]
    Gb1,
    #[serde(alias = "g1", alias = "G1")]
    G1,
    #[serde(alias = "g#1", alias = "G#1")]
    Gs1,
    #[serde(alias = "ab1", alias = "Ab1")]
    Ab1,
    #[serde(alias = "a1", alias = "A1")]
    A1,
    #[serde(alias = "a#1", alias = "A#1")]
    As1,
    #[serde(alias = "bb1", alias = "Bb1")]
    Bb1,
    #[serde(alias = "b1", alias = "B1")]
    B1,
    #[serde(alias = "c2", alias = "C2")]
    C2,
    #[serde(alias = "c#2", alias = "C#2")]
    Cs2,
    #[serde(alias = "db2", alias = "Db2")]
    Db2,
    #[serde(alias = "d2", alias = "D2")]
    D2,
    #[serde(alias = "d#2", alias = "D#2")]
    Ds2,
    #[serde(alias = "eb2", alias = "Eb2")]
    Eb2,
    #[serde(alias = "e2", alias = "E2")]
    E2,
    #[serde(alias = "f2", alias = "F2")]
    F2,
    #[serde(alias = "f#2", alias = "F#2")]
    Fs2,
    #[serde(alias = "gb2", alias = "Gb2")]
    Gb2,
    #[serde(alias = "g2", alias = "G2")]
    G2,
    #[serde(alias = "g#2", alias = "G#2")]
    Gs2,
    #[serde(alias = "ab2", alias = "Ab2")]
    Ab2,
    #[serde(alias = "a2", alias = "A2")]
    A2,
    #[serde(alias = "a#2", alias = "A#2")]
    As2,
    #[serde(alias = "bb2", alias = "Bb2")]
    Bb2,
    #[serde(alias = "b2", alias = "B2")]
    B2,
    #[serde(alias = "c3", alias = "C3")]
    C3,
    #[serde(alias = "c#3", alias = "C#3")]
    Cs3,
    #[serde(alias = "db3", alias = "Db3")]
    Db3,
    #[serde(alias = "d3", alias = "D3")]
    D3,
    #[serde(alias = "d#3", alias = "D#3")]
    Ds3,
    #[serde(alias = "eb3", alias = "Eb3")]
    Eb3,
    #[serde(alias = "e3", alias = "E3")]
    E3,
    #[serde(alias = "f3", alias = "F3")]
    F3,
    #[serde(alias = "f#3", alias = "F#3")]
    Fs3,
    #[serde(alias = "gb3", alias = "Gb3")]
    Gb3,
    #[serde(alias = "g3", alias = "G3")]
    G3,
    #[serde(alias = "g#3", alias = "G#3")]
    Gs3,
    #[serde(alias = "ab3", alias = "Ab3")]
    Ab3,
    #[serde(alias = "a3", alias = "A3")]
    A3,
    #[serde(alias = "a#3", alias = "A#3")]
    As3,
    #[serde(alias = "bb3", alias = "Bb3")]
    Bb3,
    #[serde(alias = "b3", alias = "B3")]
    B3,
    #[serde(alias = "c4", alias = "C4")]
    C4,
    #[serde(alias = "c#4", alias = "C#4")]
    Cs4,
    #[serde(alias = "db4", alias = "Db4")]
    Db4,
    #[serde(alias = "d4", alias = "D4")]
    D4,
    #[serde(alias = "d#4", alias = "D#4")]
    Ds4,
    #[serde(alias = "eb4", alias = "Eb4")]
    Eb4,
    #[serde(alias = "e4", alias = "E4")]
    E4,
    #[serde(alias = "f4", alias = "F4")]
    F4,
    #[serde(alias = "f#4", alias = "F#4")]
    Fs4,
    #[serde(alias = "gb4", alias = "Gb4")]
    Gb4,
    #[serde(alias = "g4", alias = "G4")]
    G4,
    #[serde(alias = "g#4", alias = "G#4")]
    Gs4,
    #[serde(alias = "ab4", alias = "Ab4")]
    Ab4,
    #[serde(alias = "a4", alias = "A4")]
    A4,
    #[serde(alias = "a#4", alias = "A#4")]
    As4,
    #[serde(alias = "bb4", alias = "Bb4")]
    Bb4,
    #[serde(alias = "b4", alias = "B4")]
    B4,
    #[serde(alias = "c5", alias = "C5")]
    C5,
    #[serde(alias = "c#5", alias = "C#5")]
    Cs5,
    #[serde(alias = "db5", alias = "Db5")]
    Db5,
    #[serde(alias = "d5", alias = "D5")]
    D5,
    #[serde(alias = "d#5", alias = "D#5")]
    Ds5,
    #[serde(alias = "eb5", alias = "Eb5")]
    Eb5,
    #[serde(alias = "e5", alias = "E5")]
    E5,
    #[serde(alias = "f5", alias = "F5")]
    F5,
    #[serde(alias = "f#5", alias = "F#5")]
    Fs5,
    #[serde(alias = "gb5", alias = "Gb5")]
    Gb5,
    #[serde(alias = "g5", alias = "G5")]
    G5,
    #[serde(alias = "g#5", alias = "G#5")]
    Gs5,
    #[serde(alias = "ab5", alias = "Ab5")]
    Ab5,
    #[serde(alias = "a5", alias = "A5")]
    A5,
    #[serde(alias = "a#5", alias = "A#5")]
    As5,
    #[serde(alias = "bb5", alias = "Bb5")]
    Bb5,
    #[serde(alias = "b5", alias = "B5")]
    B5,
    #[serde(alias = "c6", alias = "C6")]
    C6,
    #[serde(alias = "c#6", alias = "C#6")]
    Cs6,
    #[serde(alias = "db6", alias = "Db6")]
    Db6,
    #[serde(alias = "d6", alias = "D6")]
    D6,
    #[serde(alias = "d#6", alias = "D#6")]
    Ds6,
    #[serde(alias = "eb6", alias = "Eb6")]
    Eb6,
    #[serde(alias = "e6", alias = "E6")]
    E6,
    #[serde(alias = "f6", alias = "F6")]
    F6,
    #[serde(alias = "f#6", alias = "F#6")]
    Fs6,
    #[serde(alias = "gb6", alias = "Gb6")]
    Gb6,
    #[serde(alias = "g6", alias = "G6")]
    G6,
    #[serde(alias = "g#6", alias = "G#6")]
    Gs6,
    #[serde(alias = "ab6", alias = "Ab6")]
    Ab6,
    #[serde(alias = "a6", alias = "A6")]
    A6,
    #[serde(alias = "a#6", alias = "A#6")]
    As6,
    #[serde(alias = "bb6", alias = "Bb6")]
    Bb6,
    #[serde(alias = "b6", alias = "B6")]
    B6,
    #[serde(alias = "c7", alias = "C7")]
    C7,
    #[serde(alias = "c#7", alias = "C#7")]
    Cs7,
    #[serde(alias = "db7", alias = "Db7")]
    Db7,
    #[serde(alias = "d7", alias = "D7")]
    D7,
    #[serde(alias = "d#7", alias = "D#7")]
    Ds7,
    #[serde(alias = "eb7", alias = "Eb7")]
    Eb7,
    #[serde(alias = "e7", alias = "E7")]
    E7,
    #[serde(alias = "f7", alias = "F7")]
    F7,
    #[serde(alias = "f#7", alias = "F#7")]
    Fs7,
    #[serde(alias = "gb7", alias = "Gb7")]
    Gb7,
    #[serde(alias = "g7", alias = "G7")]
    G7,
    #[serde(alias = "g#7", alias = "G#7")]
    Gs7,
    #[serde(alias = "ab7", alias = "Ab7")]
    Ab7,
    #[serde(alias = "a7", alias = "A7")]
    A7,
    #[serde(alias = "a#7", alias = "A#7")]
    As7,
    #[serde(alias = "bb7", alias = "Bb7")]
    Bb7,
    #[serde(alias = "b7", alias = "B7")]
    B7,
    #[serde(alias = "c8", alias = "C8")]
    C8,
    #[serde(alias = "c#8", alias = "C#8")]
    Cs8,
    #[serde(alias = "db8", alias = "Db8")]
    Db8,
    #[serde(alias = "d8", alias = "D8")]
    D8,
    #[serde(alias = "d#8", alias = "D#8")]
    Ds8,
    #[serde(alias = "eb8", alias = "Eb8")]
    Eb8,
    #[serde(alias = "e8", alias = "E8")]
    E8,
    #[serde(alias = "f8", alias = "F8")]
    F8,
    #[serde(alias = "f#8", alias = "F#8")]
    Fs8,
    #[serde(alias = "gb8", alias = "Gb8")]
    Gb8,
    #[serde(alias = "g8", alias = "G8")]
    G8,
    #[serde(alias = "g#8", alias = "G#8")]
    Gs8,
    #[serde(alias = "ab8", alias = "Ab8")]
    Ab8,
    #[serde(alias = "a8", alias = "A8")]
    A8,
    #[serde(alias = "a#8", alias = "A#8")]
    As8,
    #[serde(alias = "bb8", alias = "Bb8")]
    Bb8,
    #[serde(alias = "b8", alias = "B8")]
    B8,
}

impl Into<Float> for Note {
    fn into(self) -> Float {
        match self {
            Note::C0 => 16.35,
            Note::Cs0 => 17.32,
            Note::Db0 => 17.32,
            Note::D0 => 18.35,
            Note::Ds0 => 19.45,
            Note::Eb0 => 19.45,
            Note::E0 => 20.6,
            Note::F0 => 21.83,
            Note::Fs0 => 23.12,
            Note::Gb0 => 23.12,
            Note::G0 => 24.5,
            Note::Gs0 => 25.96,
            Note::Ab0 => 25.96,
            Note::A0 => 27.5,
            Note::As0 => 29.14,
            Note::Bb0 => 29.14,
            Note::B0 => 30.87,
            Note::C1 => 32.7,
            Note::Cs1 => 34.65,
            Note::Db1 => 34.65,
            Note::D1 => 36.71,
            Note::Ds1 => 38.89,
            Note::Eb1 => 38.89,
            Note::E1 => 41.2,
            Note::F1 => 43.65,
            Note::Fs1 => 46.25,
            Note::Gb1 => 46.25,
            Note::G1 => 49.0,
            Note::Gs1 => 51.91,
            Note::Ab1 => 51.91,
            Note::A1 => 55.0,
            Note::As1 => 58.27,
            Note::Bb1 => 58.27,
            Note::B1 => 61.74,
            Note::C2 => 65.41,
            Note::Cs2 => 69.3,
            Note::Db2 => 69.3,
            Note::D2 => 73.42,
            Note::Ds2 => 77.78,
            Note::Eb2 => 77.78,
            Note::E2 => 82.41,
            Note::F2 => 87.31,
            Note::Fs2 => 92.5,
            Note::Gb2 => 92.5,
            Note::G2 => 98.0,
            Note::Gs2 => 103.83,
            Note::Ab2 => 103.83,
            Note::A2 => 110.0,
            Note::As2 => 116.54,
            Note::Bb2 => 116.54,
            Note::B2 => 123.47,
            Note::C3 => 130.81,
            Note::Cs3 => 138.59,
            Note::Db3 => 138.59,
            Note::D3 => 146.83,
            Note::Ds3 => 155.56,
            Note::Eb3 => 155.56,
            Note::E3 => 164.81,
            Note::F3 => 174.61,
            Note::Fs3 => 185.0,
            Note::Gb3 => 185.0,
            Note::G3 => 196.0,
            Note::Gs3 => 207.65,
            Note::Ab3 => 207.65,
            Note::A3 => 220.0,
            Note::As3 => 233.08,
            Note::Bb3 => 233.08,
            Note::B3 => 246.94,
            Note::C4 => 261.63,
            Note::Cs4 => 277.18,
            Note::Db4 => 277.18,
            Note::D4 => 293.66,
            Note::Ds4 => 311.13,
            Note::Eb4 => 311.13,
            Note::E4 => 329.63,
            Note::F4 => 349.23,
            Note::Fs4 => 369.99,
            Note::Gb4 => 369.99,
            Note::G4 => 392.0,
            Note::Gs4 => 415.3,
            Note::Ab4 => 415.3,
            Note::A4 => 440.0,
            Note::As4 => 466.16,
            Note::Bb4 => 466.16,
            Note::B4 => 493.88,
            Note::C5 => 523.25,
            Note::Cs5 => 554.37,
            Note::Db5 => 554.37,
            Note::D5 => 587.33,
            Note::Ds5 => 622.25,
            Note::Eb5 => 622.25,
            Note::E5 => 659.25,
            Note::F5 => 698.46,
            Note::Fs5 => 739.99,
            Note::Gb5 => 739.99,
            Note::G5 => 783.99,
            Note::Gs5 => 830.61,
            Note::Ab5 => 830.61,
            Note::A5 => 880.0,
            Note::As5 => 932.33,
            Note::Bb5 => 932.33,
            Note::B5 => 987.77,
            Note::C6 => 1046.5,
            Note::Cs6 => 1108.73,
            Note::Db6 => 1108.73,
            Note::D6 => 1174.66,
            Note::Ds6 => 1244.51,
            Note::Eb6 => 1244.51,
            Note::E6 => 1318.51,
            Note::F6 => 1396.91,
            Note::Fs6 => 1479.98,
            Note::Gb6 => 1479.98,
            Note::G6 => 1567.98,
            Note::Gs6 => 1661.22,
            Note::Ab6 => 1661.22,
            Note::A6 => 1760.0,
            Note::As6 => 1864.66,
            Note::Bb6 => 1864.66,
            Note::B6 => 1975.53,
            Note::C7 => 2093.0,
            Note::Cs7 => 2217.46,
            Note::Db7 => 2217.46,
            Note::D7 => 2349.32,
            Note::Ds7 => 2489.02,
            Note::Eb7 => 2489.02,
            Note::E7 => 2637.02,
            Note::F7 => 2793.83,
            Note::Fs7 => 2959.96,
            Note::Gb7 => 2959.96,
            Note::G7 => 3135.96,
            Note::Gs7 => 3322.44,
            Note::Ab7 => 3322.44,
            Note::A7 => 3520.0,
            Note::As7 => 3729.31,
            Note::Bb7 => 3729.31,
            Note::B7 => 3951.07,
            Note::C8 => 4186.01,
            Note::Cs8 => 4434.92,
            Note::Db8 => 4434.92,
            Note::D8 => 4698.63,
            Note::Ds8 => 4978.03,
            Note::Eb8 => 4978.03,
            Note::E8 => 5274.04,
            Note::F8 => 5587.65,
            Note::Fs8 => 5919.91,
            Note::Gb8 => 5919.91,
            Note::G8 => 6271.93,
            Note::Gs8 => 6644.88,
            Note::Ab8 => 6644.88,
            Note::A8 => 7040.0,
            Note::As8 => 7458.62,
            Note::Bb8 => 7458.62,
            Note::B8 => 7902.13,
        }
    }
}

impl Display for Note {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Note::C0 => write!(f, "C0"),
            Note::Cs0 => write!(f, "C#0"),
            Note::Db0 => write!(f, "Db0"),
            Note::D0 => write!(f, "D0"),
            Note::Ds0 => write!(f, "D#0"),
            Note::Eb0 => write!(f, "Eb0"),
            Note::E0 => write!(f, "E0"),
            Note::F0 => write!(f, "F0"),
            Note::Fs0 => write!(f, "F#0"),
            Note::Gb0 => write!(f, "Gb0"),
            Note::G0 => write!(f, "G0"),
            Note::Gs0 => write!(f, "G#0"),
            Note::Ab0 => write!(f, "Ab0"),
            Note::A0 => write!(f, "A0"),
            Note::As0 => write!(f, "A#0"),
            Note::Bb0 => write!(f, "Bb0"),
            Note::B0 => write!(f, "B0"),
            Note::C1 => write!(f, "C1"),
            Note::Cs1 => write!(f, "C#1"),
            Note::Db1 => write!(f, "Db1"),
            Note::D1 => write!(f, "D1"),
            Note::Ds1 => write!(f, "D#1"),
            Note::Eb1 => write!(f, "Eb1"),
            Note::E1 => write!(f, "E1"),
            Note::F1 => write!(f, "F1"),
            Note::Fs1 => write!(f, "F#1"),
            Note::Gb1 => write!(f, "Gb1"),
            Note::G1 => write!(f, "G1"),
            Note::Gs1 => write!(f, "G#1"),
            Note::Ab1 => write!(f, "Ab1"),
            Note::A1 => write!(f, "A1"),
            Note::As1 => write!(f, "A#1"),
            Note::Bb1 => write!(f, "Bb1"),
            Note::B1 => write!(f, "B1"),
            Note::C2 => write!(f, "C2"),
            Note::Cs2 => write!(f, "C#2"),
            Note::Db2 => write!(f, "Db2"),
            Note::D2 => write!(f, "D2"),
            Note::Ds2 => write!(f, "D#2"),
            Note::Eb2 => write!(f, "Eb2"),
            Note::E2 => write!(f, "E2"),
            Note::F2 => write!(f, "F2"),
            Note::Fs2 => write!(f, "F#2"),
            Note::Gb2 => write!(f, "Gb2"),
            Note::G2 => write!(f, "G2"),
            Note::Gs2 => write!(f, "G#2"),
            Note::Ab2 => write!(f, "Ab2"),
            Note::A2 => write!(f, "A2"),
            Note::As2 => write!(f, "A#2"),
            Note::Bb2 => write!(f, "Bb2"),
            Note::B2 => write!(f, "B2"),
            Note::C3 => write!(f, "C3"),
            Note::Cs3 => write!(f, "C#3"),
            Note::Db3 => write!(f, "Db3"),
            Note::D3 => write!(f, "D3"),
            Note::Ds3 => write!(f, "D#3"),
            Note::Eb3 => write!(f, "Eb3"),
            Note::E3 => write!(f, "E3"),
            Note::F3 => write!(f, "F3"),
            Note::Fs3 => write!(f, "F#3"),
            Note::Gb3 => write!(f, "Gb3"),
            Note::G3 => write!(f, "G3"),
            Note::Gs3 => write!(f, "G#3"),
            Note::Ab3 => write!(f, "Ab3"),
            Note::A3 => write!(f, "A3"),
            Note::As3 => write!(f, "A#3"),
            Note::Bb3 => write!(f, "Bb3"),
            Note::B3 => write!(f, "B3"),
            Note::C4 => write!(f, "C4"),
            Note::Cs4 => write!(f, "C#4"),
            Note::Db4 => write!(f, "Db4"),
            Note::D4 => write!(f, "D4"),
            Note::Ds4 => write!(f, "D#4"),
            Note::Eb4 => write!(f, "Eb4"),
            Note::E4 => write!(f, "E4"),
            Note::F4 => write!(f, "F4"),
            Note::Fs4 => write!(f, "F#4"),
            Note::Gb4 => write!(f, "Gb4"),
            Note::G4 => write!(f, "G4"),
            Note::Gs4 => write!(f, "G#4"),
            Note::Ab4 => write!(f, "Ab4"),
            Note::A4 => write!(f, "A4"),
            Note::As4 => write!(f, "A#4"),
            Note::Bb4 => write!(f, "Bb4"),
            Note::B4 => write!(f, "B4"),
            Note::C5 => write!(f, "C5"),
            Note::Cs5 => write!(f, "C#5"),
            Note::Db5 => write!(f, "Db5"),
            Note::D5 => write!(f, "D5"),
            Note::Ds5 => write!(f, "D#5"),
            Note::Eb5 => write!(f, "Eb5"),
            Note::E5 => write!(f, "E5"),
            Note::F5 => write!(f, "F5"),
            Note::Fs5 => write!(f, "F#5"),
            Note::Gb5 => write!(f, "Gb5"),
            Note::G5 => write!(f, "G5"),
            Note::Gs5 => write!(f, "G#5"),
            Note::Ab5 => write!(f, "Ab5"),
            Note::A5 => write!(f, "A5"),
            Note::As5 => write!(f, "A#5"),
            Note::Bb5 => write!(f, "Bb5"),
            Note::B5 => write!(f, "B5"),
            Note::C6 => write!(f, "C6"),
            Note::Cs6 => write!(f, "C#6"),
            Note::Db6 => write!(f, "Db6"),
            Note::D6 => write!(f, "D6"),
            Note::Ds6 => write!(f, "D#6"),
            Note::Eb6 => write!(f, "Eb6"),
            Note::E6 => write!(f, "E6"),
            Note::F6 => write!(f, "F6"),
            Note::Fs6 => write!(f, "F#6"),
            Note::Gb6 => write!(f, "Gb6"),
            Note::G6 => write!(f, "G6"),
            Note::Gs6 => write!(f, "G#6"),
            Note::Ab6 => write!(f, "Ab6"),
            Note::A6 => write!(f, "A6"),
            Note::As6 => write!(f, "A#6"),
            Note::Bb6 => write!(f, "Bb6"),
            Note::B6 => write!(f, "B6"),
            Note::C7 => write!(f, "C7"),
            Note::Cs7 => write!(f, "C#7"),
            Note::Db7 => write!(f, "Db7"),
            Note::D7 => write!(f, "D7"),
            Note::Ds7 => write!(f, "D#7"),
            Note::Eb7 => write!(f, "Eb7"),
            Note::E7 => write!(f, "E7"),
            Note::F7 => write!(f, "F7"),
            Note::Fs7 => write!(f, "F#7"),
            Note::Gb7 => write!(f, "Gb7"),
            Note::G7 => write!(f, "G7"),
            Note::Gs7 => write!(f, "G#7"),
            Note::Ab7 => write!(f, "Ab7"),
            Note::A7 => write!(f, "A7"),
            Note::As7 => write!(f, "A#7"),
            Note::Bb7 => write!(f, "Bb7"),
            Note::B7 => write!(f, "B7"),
            Note::C8 => write!(f, "C8"),
            Note::Cs8 => write!(f, "C#8"),
            Note::Db8 => write!(f, "Db8"),
            Note::D8 => write!(f, "D8"),
            Note::Ds8 => write!(f, "D#8"),
            Note::Eb8 => write!(f, "Eb8"),
            Note::E8 => write!(f, "E8"),
            Note::F8 => write!(f, "F8"),
            Note::Fs8 => write!(f, "F#8"),
            Note::Gb8 => write!(f, "Gb8"),
            Note::G8 => write!(f, "G8"),
            Note::Gs8 => write!(f, "G#8"),
            Note::Ab8 => write!(f, "Ab8"),
            Note::A8 => write!(f, "A8"),
            Note::As8 => write!(f, "A#8"),
            Note::Bb8 => write!(f, "Bb8"),
            Note::B8 => write!(f, "B8"),
        }
    }
}
