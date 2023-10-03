use super::dictionary;
use super::index::MnemonicIndex;
#[cfg(not(feature = "std"))]
use {alloc::string::String, core::fmt};
#[cfg(feature = "std")]
use {std::error::Error, std::fmt, std::string::String};

/// Language agnostic mnemonic phrase representation.
///
/// This is an handy intermediate representation of a given mnemonic
/// phrase. One can use this intermediate representation to translate
/// mnemonic from one [`Language`](./dictionary/trait.Language.html)
/// to another. **However** keep in mind that the [`Seed`](./struct.Seed.html)
/// is linked to the mnemonic string in a specific language, in a specific
/// dictionary. The [`Entropy`](./struct.Entropy.html) will be the same
/// but the resulted [`Seed`](./struct.Seed.html) will differ and all
/// the derived key of a HDWallet using the [`Seed`](./struct.Seed.html)
/// as a source to generate the root key.
///
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Mnemonics<const W: usize>([MnemonicIndex; W]);

impl<const W: usize> AsRef<[MnemonicIndex]> for Mnemonics<W> {
    fn as_ref(&self) -> &[MnemonicIndex] {
        &self.0[..]
    }
}

impl<const W: usize> From<[MnemonicIndex; W]> for Mnemonics<W> {
    fn from(v: [MnemonicIndex; W]) -> Self {
        Self(v)
    }
}

/// Error during convertion from string
#[derive(Debug, Clone)]
pub enum MnemonicError {
    /// Invalid Word in mnemonics
    WordError {
        /// index of the words having an issue
        index: usize,
        /// the error returned by the dictionary
        err: dictionary::WordNotFound,
    },
    /// Number of words does not match expectation set by the function
    InvalidWords {
        /// number of expected words
        expected_words: usize,
        /// number of words received
        got_words: usize,
    },
}

impl fmt::Display for MnemonicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WordError { index, err } => write!(f, "at {}: {}", index, err),
            Self::InvalidWords {
                expected_words,
                got_words,
            } => write!(
                f,
                "Invalid number of words, expecting {} but got {}",
                expected_words, got_words
            ),
        }
    }
}

#[cfg(feature = "std")]
impl Error for MnemonicError {}

impl<const W: usize> Mnemonics<W> {
    /// Size in bits of each element of mnemonics
    pub const BITS: usize = W * 11;

    /// get the mnemonic string representation in the given
    /// [`Language`](./dictionary/trait.Language.html).
    ///
    pub fn to_string<D>(&self, dict: &D) -> String
    where
        D: dictionary::Language,
    {
        let mut out = String::new();
        for (i, m) in self.0.iter().enumerate() {
            if i > 0 {
                out.push_str(dict.separator());
            }
            out.push_str(&m.to_word(dict))
        }
        out
    }

    /// Construct the `Mnemonics` from its string representation in the given
    /// [`Language`](./dictionary/trait.Language.html).
    ///
    pub fn from_string<D>(dic: &D, mnemonics: &str) -> Result<Self, MnemonicError>
    where
        D: dictionary::Language,
    {
        let len = mnemonics.split(dic.separator()).count();
        if len == W {
            let mut output = [MnemonicIndex(0); W];
            for (i, word) in mnemonics.split(dic.separator()).enumerate() {
                let mnemonic_index = MnemonicIndex::from_word(dic, word)
                    .map_err(|err| MnemonicError::WordError { index: i, err })?;
                output[i] = mnemonic_index;
            }
            Ok(Self(output))
        } else {
            Err(MnemonicError::InvalidWords {
                expected_words: W,
                got_words: len,
            })
        }
    }

    /// Indices iterator for each mnemonic words
    pub fn indices(&self) -> impl Iterator<Item = &MnemonicIndex> {
        self.0.iter()
    }
}
