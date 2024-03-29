use super::bits;
use super::index::*;
use super::mnemonics::*;
use cryptoxide::hashing::sha2::Sha256;

#[cfg(not(feature = "std"))]
use core::fmt;
#[cfg(feature = "std")]
use {std::error::Error, std::fmt};

/// Entropy is a random piece of data
///
/// See module documentation for mode details about how to use
/// `Entropy`.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Entropy<const N: usize>(pub [u8; N]);

/// Possible error when trying to create entropy from the mnemonics
#[derive(Debug, Clone)]
pub enum EntropyError {
    /// Invalid parameters when the function is called with mismatching bits
    InvalidParameters {
        /// number of checksum bits asked
        checksum_bits: usize,
        /// number of total bits
        total_bits: usize,
        /// number of words in mnemonics
        words: usize,
    },
    /// Mismatch in checksum
    ChecksumInvalid,
}

impl fmt::Display for EntropyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidParameters {
                checksum_bits,
                total_bits,
                words,
            } => {
                write!(
                    f,
                    "Invalid Parameters checksum-bits={}, total-bits={}, words={}",
                    checksum_bits, total_bits, words
                )
            }
            Self::ChecksumInvalid => write!(f, "Invalid Checksum"),
        }
    }
}

#[cfg(feature = "std")]
impl Error for EntropyError {}

impl<const N: usize> Entropy<N> {
    /// generate entropy using the given random generator.
    pub fn generate<G>(gen: G) -> Self
    where
        G: Fn() -> u8,
    {
        let mut bytes = [0u8; N];
        for e in bytes.iter_mut() {
            *e = gen();
        }
        Self(bytes)
    }

    fn full_checksum_data(&self) -> [u8; 32] {
        Sha256::new().update(&self.0).finalize()
    }

    /// Try to create an entropy object from the slice
    ///
    /// if the slice is not the right size, None is returned
    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() == N {
            let mut out = [0u8; N];
            out.copy_from_slice(slice);
            Some(Self(out))
        } else {
            None
        }
    }

    /// retrieve the `Entropy` from the given [`Mnemonics`](./struct.Mnemonics.html).
    ///
    /// # Example
    ///
    /// ```
    /// # use bip39_dict::{ENGLISH, Mnemonics, Entropy};
    ///
    /// const MNEMONICS : &'static str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    /// let mnemonics = Mnemonics::from_string(&ENGLISH, MNEMONICS)
    ///     .expect("validating the given mnemonics phrase");
    ///
    /// let entropy = Entropy::<16>::from_mnemonics::<12, 4>(&mnemonics)
    ///     .expect("retrieving the entropy from the mnemonics");
    /// ```
    ///
    /// # Error
    ///
    /// This function may fail if the Mnemonic has an invalid checksum. As part of the
    /// BIP39, the checksum must be embedded in the mnemonic phrase. This allow to check
    /// the mnemonics have been correctly entered by the user.
    ///
    pub fn from_mnemonics<const W: usize, const CS: usize>(
        mnemonics: &Mnemonics<W>,
    ) -> Result<Self, EntropyError> {
        assert!(CS <= 256);

        let total_bits = N * 8 + CS;
        if total_bits != Mnemonics::<W>::BITS {
            return Err(EntropyError::InvalidParameters {
                checksum_bits: CS,
                total_bits,
                words: W,
            });
        }
        use bits::BitWriterBy11;

        let mut entropy = [0u8; N];
        let mut entropy_writer_pos = 0;
        let mut checksum_data = [0u8; 256];

        // emit the byte to entropy for the N first byte, then to the checksum_data
        let emit = |b: u8| {
            if entropy_writer_pos >= N {
                checksum_data[entropy_writer_pos - N] = b;
            } else {
                entropy[entropy_writer_pos] = b;
            }
            entropy_writer_pos += 1;
        };
        let mut to_validate = BitWriterBy11::new(emit);
        for mnemonic in mnemonics.indices() {
            to_validate.write(mnemonic.0);
        }
        to_validate.finalize();

        let ret = Self(entropy);

        // check the checksum got from the mnemonics, from the one calculated
        // from the entropy generated
        let expected_checksum = ret.full_checksum_data();
        if CS > 0 {
            let checksum_data = &checksum_data[0..(entropy_writer_pos - N)];
            let mut rem = CS;
            let mut ofs = 0;
            while rem > 0 {
                if rem >= 8 {
                    if checksum_data[ofs] != expected_checksum[ofs] {
                        return Err(EntropyError::ChecksumInvalid);
                    }
                    rem -= 8;
                } else {
                    // process up to 7 bits
                    let mask = ((1 << rem) - 1) << (8 - rem);
                    if (checksum_data[ofs] & mask) != (expected_checksum[ofs] & mask) {
                        return Err(EntropyError::ChecksumInvalid);
                    }
                    rem = 0;
                }
                ofs += 1;
            }
        }

        Ok(ret)
    }

    /// convert the given `Entropy` into a mnemonic phrase of W words.
    ///
    /// # Example
    ///
    /// ```
    /// # use bip39_dict::{ENGLISH, Entropy};
    ///
    /// let entropy = Entropy::<16>([0;16]);
    ///
    /// // convert the 16 bytes entropy into 12 words with 4 bits of checksum
    /// let mnemonics = entropy.to_mnemonics::<12, 4>()
    /// 	.expect("correct value of words/checksum for 16 bytes entropy")
    ///     .to_string(&ENGLISH);
    /// assert_eq!(mnemonics, "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about");
    /// ```
    ///
    pub fn to_mnemonics<const W: usize, const CS: usize>(
        &self,
    ) -> Result<Mnemonics<W>, EntropyError> {
        assert!(CS <= 256);
        let total_bits = N * 8 + CS;
        if total_bits != Mnemonics::<W>::BITS {
            return Err(EntropyError::InvalidParameters {
                checksum_bits: CS,
                total_bits,
                words: W,
            });
        }
        use bits::{NextRead, ReadState};

        let checksum = self.full_checksum_data();

        let mut state = ReadState::default();
        let mut read_pos = 0;
        let mut write_pos = 0;

        let mut words = [MnemonicIndex(0); W];
        while write_pos < W {
            let next_byte = if read_pos >= N {
                checksum[read_pos - N]
            } else {
                self.0[read_pos]
            };
            read_pos += 1;
            match state.read8(next_byte) {
                NextRead::Zero(next_state) => {
                    state = next_state;
                }
                NextRead::One(n, next_state) => {
                    words[write_pos] = MnemonicIndex::new(n).unwrap();
                    write_pos += 1;
                    state = next_state;
                }
            }
        }

        Ok(Mnemonics::<W>::from(words))
    }
}

impl<const N: usize> AsRef<[u8]> for Entropy<N> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
