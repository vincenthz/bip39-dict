# BIP39 Dictionary

BIP39 dictionary encoding/decoding

BIP39 dictionaries gives a set of 2048 order specific words for multiple languages.

Every words represent a 11 bits index, which combined together represent a
binary encoding by adding some checksum bits (which serve as padding), this
allow to encode any arbitary data stream.

This crate can be used to produced standard BIP39 data, but also give the ability to
relax the standard with more capabilities.

The maximum length allowed for the checksum is 32 bytes (256 bits), and the
checksum used is currently always SHA2-256. At later point this could be made
customizable as well.

The following relation need to hold for having a valid decoding/encoding:

```
length_bytes(data) * 8 + checksum = number_of(words) * 11
```

The standard BIP39 encoding use the following value:

| Words | In Bits | Full Bytes | Checksum Bits |
| ----- | ------- | ---------- | ------------- |
| 12    | 132     | 16 (128)   | 4             |
| 15    | 165     | 20 (160)   | 5             |
| 18    | 198     | 24 (192)   | 6             |
| 21    | 231     | 28 (224)   | 7             |
| 24    | 264     | 32 (256)   | 8             |

But by relaxing this BIP39 standard, we can make the checksum
variable and all combinaison of words valid:

| Words | In Bits | Full Bytes | Remaining Bits | Standard |
| ----- | ------- | ---------- | -------------- | -------- |
| 1     | 11      | 1 (8)      | 3              | no       |
| 2     | 22      | 2 (16)     | 6              | no       |
| 3     | 33      | 4 (32)     | 1              | no       |
| 4     | 44      | 5 (40)     | 4              | no       |
| 5     | 55      | 6 (48)     | 7              | no       |
| 6     | 66      | 8 (64)     | 2              | no       |
| 7     | 77      | 9 (72)     | 5              | no       |
| 8     | 88      | 11 (88)    | 0              | no       |
| 9     | 99      | 12 (96)    | 3              | yes      |
| 10    | 110     | 13 (104)   | 6              | no       |
| 11    | 121     | 15 (120)   | 1              | no       |
| 12    | 132     | 16 (128)   | 4              | yes      |
| ...   | ...     | ...        | ...            | no       |
| ...   | ...     | ...        | ...            | no       |
| 15    | 165     | 20 (160)   | 5              | yes      |
| ...   | ...     | ...        | ...            | no       |
| ...   | ...     | ...        | ...            | no       |
| 18    | 198     | 24 (192)   | 6              | yes      |
| ...   | ...     | ...        | ...            | no       |
| ...   | ...     | ...        | ...            | no       |
| 21    | 231     | 28 (224)   | 7              | yes      |
| ...   | ...     | ...        | ...            | no       |
| ...   | ...     | ...        | ...            | no       |
| 24    | 264     | 32 (256)   | 8              | yes      |
| 24    | 264     | 33 (264)   | 0              | no       |
| ...   | ...     | ...        | ...            | no       |
| ...   | ...     | ...        | ...            | no       |
| 27    | 297     | 37 (296)   | 1              | no       |


# Thanks

This crate has been heavily inspired by the BIP39 implementation at
[rust-cardano bip39](https://github.com/input-output-hk/rust-cardano/blob/master/cardano/src/bip/bip39.rs)
and dictionary data and test vectors have been lifted from this source.
