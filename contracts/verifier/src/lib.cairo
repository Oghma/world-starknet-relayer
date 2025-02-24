pub mod groth16_verifier;
mod groth16_verifier_constants;
pub mod universal_ecip;
pub mod world_relayer_verifier;
use core::num::traits::{Bounded, WideMul};

#[derive(Drop, Debug, Copy, PartialEq, Serde)]
pub struct Journal {
    pub latest_block: u64,
    pub state_root: u256,
}

pub fn decode_journal(journal_bytes: Span<u8>) -> Journal {
    let mut offset = 0; // Skip initial bytes

    // Parse latest_block
    let mut latest_block: u64 = 0;
    let mut i = 0;
    while i < 8 {
        let f0: u64 = (*journal_bytes.at(offset + i)).into();
        let f1: u64 = BitShift::shl(f0, 8 * i.into());
        latest_block += f1;
        i += 1;
    };

    // Parse latest_root
    offset += 8;
    offset += 4; // Skip length indicator (66, 0, 0, 0)
    let mut state_root: u256 = 0;
    let mut i = 0;

    loop {
        if i >= 32 {
            break;
        }
        // Read each byte and shift into position
        let byte: u256 = (*journal_bytes.at(offset + i)).into();
        state_root = state_root * 256 + byte;
        i += 1;
    };

    Journal { latest_block, state_root }
}

trait BitShift<T> {
    fn shl(x: T, n: T) -> T;
    fn shr(x: T, n: T) -> T;
}

impl U256BitShift of BitShift<u256> {
    fn shl(x: u256, n: u256) -> u256 {
        let res = WideMul::wide_mul(x, pow(2, n));
        u256 { low: res.limb0, high: res.limb1 }
    }

    fn shr(x: u256, n: u256) -> u256 {
        x / pow(2, n)
    }
}

impl U64BitShift of BitShift<u64> {
    fn shl(x: u64, n: u64) -> u64 {
        (WideMul::wide_mul(x, pow(2, n)) & Bounded::<u64>::MAX.into()).try_into().unwrap()
    }

    fn shr(x: u64, n: u64) -> u64 {
        x / pow(2, n)
    }
}

impl U128BitShift of BitShift<u128> {
    fn shl(x: u128, n: u128) -> u128 {
        let res = WideMul::wide_mul(x, pow(2, n));
        res.low
    }

    fn shr(x: u128, n: u128) -> u128 {
        x / pow(2, n)
    }
}

fn pow<T, +Sub<T>, +Mul<T>, +Div<T>, +Rem<T>, +PartialEq<T>, +Into<u8, T>, +Drop<T>, +Copy<T>>(
    base: T, exp: T,
) -> T {
    if exp == 0_u8.into() {
        1_u8.into()
    } else if exp == 1_u8.into() {
        base
    } else if exp % 2_u8.into() == 0_u8.into() {
        pow(base * base, exp / 2_u8.into())
    } else {
        base * pow(base * base, exp / 2_u8.into())
    }
}

#[cfg(test)]
mod tests {
    use super::decode_journal;

    #[test]
    fn decode_journal_test() {
        let journal_bytes = get_journal_bytes();

        let journal = decode_journal(journal_bytes);
        assert_eq!(journal.latest_block, 21891875);
        assert_eq!(
            journal.state_root,
            17535143312471158466661076185880618200719072926547797113181234216764225486579,
        );
    }

    fn get_journal_bytes() -> Span<u8> {
        array![
            35,
            11,
            78,
            1,
            0,
            0,
            0,
            0,
            32,
            0,
            0,
            0,
            38,
            196,
            138,
            22,
            71,
            45,
            64,
            179,
            97,
            81,
            85,
            93,
            19,
            63,
            51,
            139,
            240,
            221,
            140,
            41,
            94,
            168,
            193,
            21,
            133,
            129,
            232,
            74,
            11,
            109,
            254,
            243,
        ]
            .span()
    }
}
