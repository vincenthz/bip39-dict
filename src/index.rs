use super::dictionary;

/// the maximum authorized value for a mnemonic. i.e. 2047
pub const MAX_MNEMONIC_VALUE: u16 = 2047;

/// Safe representation of a valid mnemonic index (see
/// [`MAX_MNEMONIC_VALUE`](./constant.MAX_MNEMONIC_VALUE.html)).
///
/// See [`dictionary module documentation`](./dictionary/index.html) for
/// more details about how to use this.
///
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct MnemonicIndex(pub u16);

impl MnemonicIndex {
    /// smart constructor, validate the given value fits the mnemonic index
    /// boundaries (see [`MAX_MNEMONIC_VALUE`](./constant.MAX_MNEMONIC_VALUE.html)).
    pub fn new(m: u16) -> Option<Self> {
        if m <= MAX_MNEMONIC_VALUE {
            Some(MnemonicIndex(m))
        } else {
            None
        }
    }

    /// lookup in the given dictionary to retrieve the mnemonic word.
    pub fn to_word<D>(self, dict: &D) -> &'static str
    where
        D: dictionary::Language,
    {
        dict.lookup_word(self)
    }

    /// retrieve the Mnemonic index from the given word in the
    /// given dictionary.
    ///
    /// # Error
    ///
    /// May fail with a [`LanguageError`](enum.Error.html#variant.LanguageError)
    /// if the given [`Language`](./dictionary/trait.Language.html) returns the
    /// given word is not within its dictionary.
    ///
    pub fn from_word<D>(dict: &D, word: &str) -> Result<Self, dictionary::WordNotFound>
    where
        D: dictionary::Language,
    {
        let v = dict.lookup_mnemonic(word)?;
        Ok(v)
    }
}
