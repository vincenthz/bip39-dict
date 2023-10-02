//! Language support for BIP39 implementations.
//!
//! We provide default dictionaries  for the some common languages.
//! This interface is exposed to allow users to implement custom
//! dictionaries.
//!
//! Due to keeping the depedencies as small as possible, we do not
//! support UTF8 NFKD by default. Users must be sure to compose (or decompose)
//! our output (or input) UTF8 strings.
#[cfg(feature = "cjk")]
mod chinese_simplified;
#[cfg(feature = "cjk")]
mod chinese_traditional;
#[cfg(feature = "english")]
mod english;
#[cfg(feature = "latin")]
mod french;
#[cfg(feature = "latin")]
mod italian;
#[cfg(feature = "cjk")]
mod japanese;
#[cfg(feature = "cjk")]
mod korean;
#[cfg(feature = "latin")]
mod spanish;

#[cfg(not(feature = "std"))]
use {
    alloc::string::{String, ToString},
    core::fmt,
};

#[cfg(feature = "std")]
use {
    std::error::Error,
    std::fmt,
    std::string::{String, ToString},
};

use crate::index::MnemonicIndex;

/// Errors associated to a given language/dictionary
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct WordNotFound {
    pub word_searched: String,
}

impl fmt::Display for WordNotFound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "word '{}' not found in dictionary", self.word_searched)
    }
}

#[cfg(feature = "std")]
impl Error for WordNotFound {}

/// trait to represent the the properties that needs to be associated to
/// a given language and its dictionary of known mnemonic words.
///
pub trait Language {
    fn name(&self) -> &'static str;
    fn separator(&self) -> &'static str;
    fn lookup_mnemonic(&self, word: &str) -> Result<MnemonicIndex, WordNotFound>;
    fn lookup_word(&self, mnemonic: MnemonicIndex) -> &'static str;
}

/// Default Dictionary basic support for the different main languages.
/// This dictionary expect the inputs to have been normalized (UTF-8 NFKD).
///
/// If you wish to implement support for non pre-normalized form you can
/// create reuse this dictionary in a custom struct and implement support
/// for [`Language`](./trait.Language.html) accordingly (_hint_: use
/// [`unicode-normalization`](https://crates.io/crates/unicode-normalization)).
///
pub struct DefaultDictionary {
    pub words: [&'static str; 2048],
    pub name: &'static str,
    pub ordered: bool,
}
impl Language for DefaultDictionary {
    fn name(&self) -> &'static str {
        self.name
    }
    fn separator(&self) -> &'static str {
        " "
    }
    fn lookup_mnemonic(&self, word: &str) -> Result<MnemonicIndex, WordNotFound> {
        if self.ordered {
            match self.words.binary_search(&word) {
                Ok(v) => Ok(MnemonicIndex::new(v as u16).unwrap()),
                Err(_) => Err(WordNotFound {
                    word_searched: word.to_string(),
                }),
            }
        } else {
            match self.words.iter().position(|x| x == &word) {
                None => Err(WordNotFound {
                    word_searched: word.to_string(),
                }),
                Some(v) => {
                    Ok(
                        // it is safe to call unwrap as we guarantee that the
                        // returned index `v` won't be out of bound for a
                        // `MnemonicIndex` (DefaultDictionary.words is an array of 2048 elements)
                        MnemonicIndex::new(v as u16).unwrap(),
                    )
                }
            }
        }
    }

    fn lookup_word(&self, mnemonic: MnemonicIndex) -> &'static str {
        self.words[mnemonic.0 as usize]
    }
}

/// default English dictionary as provided by the
/// [BIP39 standard](https://github.com/bitcoin/bips/blob/master/bip-0039/bip-0039-wordlists.md#wordlists)
///
#[cfg(feature = "english")]
pub const ENGLISH: DefaultDictionary = DefaultDictionary {
    words: english::WORDS,
    name: "english",
    ordered: true,
};

/// default French dictionary as provided by the
/// [BIP39 standard](https://github.com/bitcoin/bips/blob/master/bip-0039/bip-0039-wordlists.md#french)
///
#[cfg(feature = "latin")]
pub const FRENCH: DefaultDictionary = DefaultDictionary {
    words: french::WORDS,
    name: "french",
    ordered: false,
};

/// default Japanese dictionary as provided by the
/// [BIP39 standard](https://github.com/bitcoin/bips/blob/master/bip-0039/bip-0039-wordlists.md#japanese)
///
#[cfg(feature = "cjk")]
pub const JAPANESE: DefaultDictionary = DefaultDictionary {
    words: japanese::WORDS,
    name: "japanese",
    ordered: false,
};

/// default Korean dictionary as provided by the
/// [BIP39 standard](https://github.com/bitcoin/bips/blob/master/bip-0039/bip-0039-wordlists.md#japanese)
///
#[cfg(feature = "cjk")]
pub const KOREAN: DefaultDictionary = DefaultDictionary {
    words: korean::WORDS,
    name: "korean",
    ordered: true,
};

/// default chinese simplified dictionary as provided by the
/// [BIP39 standard](https://github.com/bitcoin/bips/blob/master/bip-0039/bip-0039-wordlists.md#chinese)
///
#[cfg(feature = "cjk")]
pub const CHINESE_SIMPLIFIED: DefaultDictionary = DefaultDictionary {
    words: chinese_simplified::WORDS,
    name: "chinese-simplified",
    ordered: false,
};
/// default chinese traditional dictionary as provided by the
/// [BIP39 standard](https://github.com/bitcoin/bips/blob/master/bip-0039/bip-0039-wordlists.md#chinese)
///
#[cfg(feature = "cjk")]
pub const CHINESE_TRADITIONAL: DefaultDictionary = DefaultDictionary {
    words: chinese_traditional::WORDS,
    name: "chinese-traditional",
    ordered: false,
};

/// default italian dictionary as provided by the
/// [BIP39 standard](https://github.com/bitcoin/bips/blob/master/bip-0039/bip-0039-wordlists.md#italian)
///
#[cfg(feature = "latin")]
pub const ITALIAN: DefaultDictionary = DefaultDictionary {
    words: italian::WORDS,
    name: "italian",
    ordered: true,
};

/// default spanish dictionary as provided by the
/// [BIP39 standard](https://github.com/bitcoin/bips/blob/master/bip-0039/bip-0039-wordlists.md#spanish)
///
#[cfg(feature = "latin")]
pub const SPANISH: DefaultDictionary = DefaultDictionary {
    words: spanish::WORDS,
    name: "spanish",
    ordered: false,
};

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! dict_valid {
        ($dict:ident) => {{
            assert_eq!($dict.words.windows(2).all(|w| w[0] <= w[1]), $dict.ordered);

            for (i, word) in $dict.words.iter().enumerate() {
                assert_eq!(
                    $dict.lookup_mnemonic(word),
                    Ok(MnemonicIndex::new(i as u16).unwrap())
                );
            }
        }};
    }

    #[test]
    fn dict_valid() {
        #[cfg(feature = "english")]
        dict_valid!(ENGLISH);

        #[cfg(feature = "latin")]
        {
            dict_valid!(FRENCH);
            dict_valid!(ITALIAN);
            dict_valid!(SPANISH);
        }
        #[cfg(feature = "cjk")]
        {
            dict_valid!(CHINESE_SIMPLIFIED);
            dict_valid!(CHINESE_TRADITIONAL);
            dict_valid!(JAPANESE);
            dict_valid!(KOREAN);
        }
    }
}
