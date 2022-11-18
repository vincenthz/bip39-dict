use super::bits;
use super::index::*;
use super::mnemonics::*;
use cryptoxide::hashing::sha2::Sha256;

#[cfg(not(feature = "std"))]
use {
    alloc::vec::Vec,
    core::fmt,
};
#[cfg(feature = "std")]
use {
    std::vec::Vec,
    std::error::Error,
    std::fmt,
};

/// Entropy is a random piece of data
///
/// See module documentation for mode details about how to use
/// `Entropy`.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Entropy<const N: usize>(pub [u8; N]);

#[derive(Debug, Clone)]
pub enum EntropyError {
    InvalidParameters {
        checksum_bits: usize,
        total_bits: usize,
        words: usize,
    },
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

        let mut to_validate = BitWriterBy11::new();
        for mnemonic in mnemonics.indices() {
            to_validate.write(mnemonic.0);
        }

        let mut entropy = [0u8; N];
        let r = to_validate.to_bytes();
        entropy.copy_from_slice(&r[0..N]);
        let ret = Self(entropy);

        // check the checksum got from the mnemonics, from the one calculated
        // from the entropy generated
        let expected_checksum = ret.full_checksum_data();
        if CS > 0 {
            let checksum_data = &r[N..]; // checksum data computed is in r, after the N bytes of entropy
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
        use bits::BitReaderBy11;

        let mut combined = Vec::from(self.as_ref());
        combined.extend_from_slice(&self.full_checksum_data());

        let mut reader = BitReaderBy11::new(&combined);

        let mut words = [MnemonicIndex(0); W];
        for i in 0..W {
            let n = reader.read();
            words[i] = MnemonicIndex::new(n).unwrap();
        }

        Ok(Mnemonics::<W>::from(words))
    }
}

impl<const N: usize> AsRef<[u8]> for Entropy<N> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
