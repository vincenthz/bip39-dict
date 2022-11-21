#[allow(dead_code)]
const NUM_BITS_PER_BLOCK: usize = 11;

// this represent the number of partial bits
//
// 11 bits writer to bytes:
//
// current              then
// partial bits         full byte   left over
// 0            + 11 => 1*8       + 3          => S3
// 1            + 11 => 1*8       + 4          => S4
// 2            + 11 => 1*8       + 5          => S5
// 3            + 11 => 1*8       + 6          => S6
// 4            + 11 => 1*8       + 7          => S7
// 5            + 11 => 2*8       + 0          => S0
// 6            + 11 => 2*8       + 1          => S1
// 7            + 11 => 2*8       + 2          => S2
//
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum WriteState {
    S0,
    S1(u8),
    S2(u8),
    S3(u8),
    S4(u8),
    S5(u8),
    S6(u8),
    S7(u8),
}

enum NextWrite {
    One(u8, WriteState),
    Double(u8, u8, WriteState),
}

impl WriteState {
    // append 11 bits to the state and create a next state
    pub fn append11(self, v: u16) -> NextWrite {
        assert!(v < 2048);
        match self {
            WriteState::S0 => {
                NextWrite::One((v >> 3) as u8, WriteState::S3((v & 0b0000_0111) as u8))
            }
            WriteState::S1(c) => NextWrite::One(
                (c << 7) | (v >> 4) as u8,
                WriteState::S4((v & 0b0000_1111) as u8),
            ),
            WriteState::S2(c) => NextWrite::One(
                (c << 6) | (v >> 5) as u8,
                WriteState::S5((v & 0b0001_1111) as u8),
            ),
            WriteState::S3(c) => NextWrite::One(
                (c << 5) | (v >> 6) as u8,
                WriteState::S6((v & 0b0011_1111) as u8),
            ),
            WriteState::S4(c) => NextWrite::One(
                (c << 4) | (v >> 7) as u8,
                WriteState::S7((v & 0b0111_1111) as u8),
            ),
            WriteState::S5(c) => {
                // 5 + 11 = 16 => 2 bytes + 0
                NextWrite::Double((c << 3) | (v >> 8) as u8, v as u8, WriteState::S0)
            }
            WriteState::S6(c) => NextWrite::Double(
                (c << 2) | (v >> 9) as u8,
                (v >> 1) as u8,
                WriteState::S1(v as u8 & 0b0000_0001),
            ),
            WriteState::S7(c) => NextWrite::Double(
                (c << 1) | (v >> 10) as u8,
                (v >> 2) as u8,
                WriteState::S2(v as u8 & 0b0000_0011),
            ),
        }
    }
}

// 8 bits reader to 11 bits
//
// current              then
// partial bits         full 11 bits   left over
// 0            + 8 => 0*11          + 8          => S8
// 1            + 8 => 0*11          + 9          => S9
// 2            + 8 => 0*11          + 10         => S10
// 3            + 8 => 1*11          + 0          => S0
// 4            + 8 => 1*11          + 1          => S1
// 5            + 8 => 1*11          + 2          => S2
// 6            + 8 => 1*11          + 3          => S3
// 7            + 8 => 1*11          + 4          => S4
// 8            + 8 => 1*11          + 5          => S5
// 9            + 8 => 1*11          + 6          => S6
// 10           + 8 => 1*11          + 7          => S7
//
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub(crate) enum ReadState {
    S0,
    S1(u16),
    S2(u16),
    S3(u16),
    S4(u16),
    S5(u16),
    S6(u16),
    S7(u16),
    S8(u16),
    S9(u16),
    S10(u16),
}

impl Default for ReadState {
    fn default() -> Self {
        ReadState::S0
    }
}

pub(crate) enum NextRead {
    Zero(ReadState),
    One(u16, ReadState),
}

impl ReadState {
    #[inline]
    fn value_or(self, b: u8) -> u32 {
        // use 32 bits as S9 (9+8) and S10 (10+8) will overflow 16 bits
        match self {
            Self::S0 => b as u32,
            Self::S1(c)
            | Self::S2(c)
            | Self::S3(c)
            | Self::S4(c)
            | Self::S5(c)
            | Self::S6(c)
            | Self::S7(c)
            | Self::S8(c)
            // those 2 overflows overflow the u16
            | Self::S9(c)
            | Self::S10(c) => (c as u32) << 8 | (b as u32),
        }
    }

    pub fn read8(self, v: u8) -> NextRead {
        let nc = self.value_or(v);
        match self {
            Self::S0 => NextRead::Zero(ReadState::S8(nc as u16)),
            Self::S1(_) => NextRead::Zero(ReadState::S9(nc as u16)),
            Self::S2(_) => NextRead::Zero(ReadState::S10(nc as u16)),
            Self::S3(_) => NextRead::One(nc as u16, ReadState::S0),
            Self::S4(_) => NextRead::One((nc >> 1) as u16, ReadState::S1(nc as u16 & 0b0001)),
            Self::S5(_) => NextRead::One((nc >> 2) as u16, ReadState::S2(nc as u16 & 0b0011)),
            Self::S6(_) => NextRead::One((nc >> 3) as u16, ReadState::S3(nc as u16 & 0b0111)),
            Self::S7(_) => NextRead::One((nc >> 4) as u16, ReadState::S4(nc as u16 & 0b1111)),
            Self::S8(_) => NextRead::One((nc >> 5) as u16, ReadState::S5(nc as u16 & 0b0001_1111)),
            Self::S9(_) => NextRead::One((nc >> 6) as u16, ReadState::S6(nc as u16 & 0b0011_1111)),
            Self::S10(_) => NextRead::One((nc >> 7) as u16, ReadState::S7(nc as u16 & 0b0111_1111)),
        }
    }
}

pub struct BitWriterBy11<F> {
    writer: F,
    state: WriteState,
}

impl<F> BitWriterBy11<F>
where
    F: FnMut(u8) -> (),
{
    pub fn new(writer: F) -> Self {
        BitWriterBy11 {
            writer,
            state: WriteState::S0,
        }
    }

    pub fn finalize(mut self) {
        match self.state {
            WriteState::S0 => {}
            WriteState::S1(c) => self.emit(c << 7),
            WriteState::S2(c) => self.emit(c << 6),
            WriteState::S3(c) => self.emit(c << 5),
            WriteState::S4(c) => self.emit(c << 4),
            WriteState::S5(c) => self.emit(c << 3),
            WriteState::S6(c) => self.emit(c << 2),
            WriteState::S7(c) => self.emit(c << 1),
        }
    }

    fn emit(&mut self, byte: u8) {
        (self.writer)(byte)
    }

    // write 11 bits in the buffer
    pub fn write(&mut self, e: u16) {
        assert!(e < 2048);

        match self.state.append11(e) {
            NextWrite::One(byte, state) => {
                self.emit(byte);
                self.state = state;
            }
            NextWrite::Double(byte1, byte2, state) => {
                self.emit(byte1);
                self.emit(byte2);
                self.state = state;
            }
        }
    }
}

#[cfg(test)]
pub struct BitReaderBy11<'a> {
    buffer: &'a [u8],
    state: ReadState,
}

#[cfg(test)]
impl<'a> BitReaderBy11<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        BitReaderBy11 {
            buffer: bytes,
            state: ReadState::S0,
        }
    }

    // read 1 or 2 bytes and returns an 11 bits unsigned integer (in shape of a u16)
    pub fn read(&mut self) -> u16 {
        let v = self.buffer[0];
        match self.state.read8(v) {
            // if it's zero, then we need to read another byte to make an 11 bits
            NextRead::Zero(next) => {
                let v2 = self.buffer[1];
                self.buffer = &self.buffer[2..];
                match next.read8(v2) {
                    // it's *guarantee* that reading 2 bytes will lead to at least 1 consumption of 16 bits
                    NextRead::Zero(_) => unreachable!(),
                    NextRead::One(r, next2) => {
                        self.state = next2;
                        r
                    }
                }
            }
            NextRead::One(r, next) => {
                self.state = next;
                self.buffer = &self.buffer[1..];
                r
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const WORDS: &'static [u16] = &[
        0b000_0000_0001,
        0b000_0000_0001,
        0b000_0000_0001,
        0b000_0000_0001,
        0b000_0000_0001,
        0b000_0000_0001,
        0b000_0000_0001,
        0b000_0000_0001,
        0b100_0000_0001,
        0b100_0000_0001,
        0b100_0000_0001,
        0b100_0000_0001,
        0b100_0000_0001,
        0b100_0000_0001,
        0b100_0000_0001,
        0b100_0000_0001,
    ];

    const BYTES: [u8; 22] = [
        0b0000_0000, // 0   - 0
        0b0010_0000, // 8   - 0 (8)
        0b0000_0100, // 16  - 1 (5)
        0b0000_0000, // 24  - 2 (2)
        0b1000_0000, // 32  - 2 (10)
        0b0001_0000, // 40  - 3 (7)
        0b0000_0010, // 48  - 4 (4)
        0b0000_0000, // 56  - 5 (1)
        0b0100_0000, // 64  - 5 (9)
        0b0000_1000, // 72  - 6 (6)
        0b0000_0001, // 80  - 7 (3)
        0b1000_0000, // 88  - 8 (0)
        0b0011_0000, // 96  - 8 (8)
        0b0000_0110, // 104 - 9 (5)
        0b0000_0000, // 112 - 10 (2)
        0b1100_0000, // 120 - 10 (10)
        0b0001_1000, // 128 - 11 (7)
        0b0000_0011, // 136 -
        0b0000_0000, // 144 -
        0b0110_0000, // 152 -
        0b0000_1100, // 160 -
        0b0000_0001, // 168 -
    ];

    #[test]
    fn bit_write_by_11() {
        let mut bytes = [0; BYTES.len()];
        let mut bytes_pos = 0;
        let emit = |b: u8| {
            bytes[bytes_pos] = b;
            bytes_pos += 1;
        };
        let mut writer = BitWriterBy11::new(emit);
        for w in WORDS {
            writer.write(*w)
        }
        writer.finalize();

        assert_eq!(bytes, BYTES);
    }

    #[test]
    fn bit_read_by_11() {
        let mut reader = BitReaderBy11::new(&BYTES);

        for (ith, w) in WORDS.iter().enumerate() {
            let word = reader.read();
            assert_eq!(word, *w, "{} WORD not correct", ith);
        }
    }
}
