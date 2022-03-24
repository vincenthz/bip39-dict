//! BIP39 Seed making
//!
//! Implement the procedure that takes BIP39 mnemonic words
//! and turn into an output bytes (used as a wallet root secret key)
//! using PBKDF2-HMAC-SHA512 combining an optional password.
//!
//! seed = PBKDF2-HMAC-SHA512(salt = "mnemonic" || password, key="abandon abandon ... about");
//!
//! The output size and the number of iteration are both configurable,
//! and the original BIP39 values are iteration=2048 and output-size=64 bytes.

use cryptoxide::hmac::Hmac;
use cryptoxide::pbkdf2::pbkdf2;
use cryptoxide::sha2::Sha512;

use super::dictionary;
use super::mnemonics::Mnemonics;

/// get the seed from the given [`Mnemonics`] and the given password.
///
/// Note that the `Seed` is not generated from the `Entropy` directly, but from the
/// render mnemonic string in a specific language (defined by the dictionary).
/// It is a design choice of Bip39.
///
/// # Safety
///
/// While it is possible to not use a password, it is recommended for protecting the seed.
///
/// # Example
///
/// ```
/// # use bip39_dict::{ENGLISH, Mnemonics, seed_from_mnemonics};
///
/// const MNEMONICS : &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
/// let mnemonics = Mnemonics::<12>::from_string(&ENGLISH, MNEMONICS)
///     .expect("valid Mnemonic phrase");
///
/// let seed : [u8; 64] = seed_from_mnemonics(&ENGLISH, &mnemonics, b"My Password", 2048);
/// ```
///
pub fn seed_from_mnemonics<D: dictionary::Language, const W: usize, const OUTPUT: usize>(
    dict: &D,
    mnemonics: &Mnemonics<W>,
    password: &[u8],
    iter: u32,
) -> [u8; OUTPUT] {
    let mut salt = Vec::from("mnemonic".as_bytes());
    salt.extend_from_slice(password);
    let mut mac = Hmac::new(Sha512::new(), mnemonics.to_string(dict).as_bytes());
    let mut result = [0; OUTPUT];
    pbkdf2(&mut mac, &salt, iter, &mut result);
    result
}
