//! BIP39 dictionary
//!
//! This can be used to convert arbitrary binary to a sequence of words (from the BIP39 wordlist)
//! with a user-chosen checksum.
//!
//! The API can be used to generate the exact same procedure as BIP39 document, and
//! can be used to generate a standard BIP39 hdwallet root secret key.
//!
//! It also extends the available procedure to allow user to use an arbitrary number of words,
//! chose different size checksum, and finally for seed making tweak the number of iteration and
//! output size to the user need.
//!
//! For more details about the BIP39 protocol, see
//! [Bitcoin Improvement Proposal 39](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki)
//!
//! # Rules of encoding
//!
//! Each BIP39 word represent 11 bits, and the scheme can encode any byte size.
//!
//! The data are all multiple of 8 bits, aka bytestream, and
//! the number of words represent a multiple of 11 bits. The checksum
//! is the difference of the two and thus the following relation need
//! to hold for:
//!
//! ```text
//! data.len() * 8 + checksum = words.len() * 11
//! ```
//!
//! The following are examples of valid encoding:
//!
//! * 1 byte data (8 bits) + 3 bits checksum = 1 word (11 bits)
//! * 2 bytes data (16 bits) + 6 bits checksum = 2 words (22 bits)
//! * 2 bytes data (16 bits) + 17 bits checksum = 3 words (33 bits)
//!
//! # Example
//!
//! # Convert an arbitrary 48 bytes entropy value to mnemonics
//!
//! ```
//! use bip39_dict::{Entropy, ENGLISH, seed_from_mnemonics};
//! // Entropy is a 48-bytes value
//! # let entropy : Entropy<48> = Entropy([0; 48]);
//!
//! // create a 36 words mnemonics with 12 bits checksum
//! let mnemonics = entropy.to_mnemonics::<36, 12>().unwrap();
//! let mnemonics_string = mnemonics.to_string(&ENGLISH);
//! ```
//1
//! ## To create a new HDWallet root secret key
//!
//!
//! ```
//! use bip39_dict::{Entropy, ENGLISH, seed_from_mnemonics};
//!
//! // first, you need to generate some entropy (dummy)
//! let entropy = Entropy::<16>::generate(|| 1);
//!
//! // human readable mnemonics (in English) to retrieve the original entropy
//! // and eventually recover a HDWallet.
//! let mnemonics = entropy.to_mnemonics::<12, 4>().unwrap();
//!
//! // The seed of the Wallet is generated from the mnemonic string
//! // in the associated language.
//! let seed: [u8; 64] = seed_from_mnemonics(&ENGLISH, &mnemonics, b"some password", 2048);
//! ```
//!

#![no_std]

#[cfg(feature = "std")]
extern crate std;

#[cfg(not(feature = "std"))]
extern crate alloc;


mod bits;
mod dictionary;
mod entropy;
mod index;
mod mnemonics;
mod seed;

pub use dictionary::*;
pub use entropy::{Entropy, EntropyError};
pub use index::MnemonicIndex;
pub use mnemonics::{MnemonicError, Mnemonics};
pub use seed::seed_from_mnemonics;

#[cfg(test)]
mod tests;
