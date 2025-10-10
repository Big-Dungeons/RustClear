#[inline(always)]
pub const fn rapidhash_nano(seed: u64, bytes: &[u8]) -> u64 {
    rapidhash_nano_core(seed, bytes)
}

#[inline(always)]
const fn rapidhash_nano_core(mut seed: u64, bytes: &[u8]) -> u64 {
    let mut a = 0;
    let mut b = 0;

    let remaining;
    if bytes.len() <= 16 {
        if bytes.len() >= 4 {
            seed ^= bytes.len() as u64;
            if bytes.len() >= 8 {
                a = read_u64(bytes, 0);
                b = read_u64(bytes, bytes.len() - 8);
            } else {
                b = read_u32(bytes, 0) as u64;
                a = read_u32(bytes, bytes.len() - 4) as u64;
            }
        } else if !bytes.is_empty() {
            a = ((bytes[0] as u64) << 45) | bytes[bytes.len() - 1] as u64;
            b = bytes[bytes.len() >> 1] as u64;
        }
        remaining = bytes.len();
    } else {
        let mut slice = bytes;
        if slice.len() > 48 {
            let mut see1 = seed;
            let mut see2 = seed;

            while slice.len() > 48 {
                seed = mix(read_u64(slice, 0) ^ RH1, read_u64(slice, 8) ^ seed);
                see1 = mix(read_u64(slice, 16) ^ RH2, read_u64(slice, 24) ^ see1);
                see2 = mix(read_u64(slice, 32) ^ RH3, read_u64(slice, 40) ^ see2);
                slice = slice.split_at(48).1;
            }

            seed ^= see1;
            seed ^= see2;
        }

        if slice.len() > 16 {
            seed = mix(read_u64(slice, 0) ^ RH3, read_u64(slice, 8) ^ seed);
            if slice.len() > 32 {
                seed = mix(read_u64(slice, 16) ^ RH3, read_u64(slice, 24) ^ seed);
            }
        }

        a = read_u64(bytes, bytes.len() - 16) ^ slice.len() as u64;
        b = read_u64(bytes, bytes.len() - 8);
        remaining = slice.len()
    }

    finish(a, b, seed, remaining)
}

#[inline(always)]
pub(super) const fn finish(mut a: u64, mut b: u64, seed: u64, remaining: usize) -> u64 {
    (a, b) = mum(a ^ RH2, b ^ seed);
    mix(a ^ RH8, b ^ RH1 ^ remaining as u64)
}

#[inline(always)]
const fn read_u64(slice: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(*slice.split_at(offset).1.first_chunk::<8>().unwrap())
}

#[inline(always)]
const fn read_u32(slice: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(*slice.split_at(offset).1.first_chunk::<4>().unwrap())
}

#[inline(always)]
const fn mum(a: u64, b: u64) -> (u64, u64) {
    let r = (a as u128).wrapping_mul(b as u128);

    (r as u64, (r >> 64) as u64)
}

#[inline(always)]
pub(super) const fn mix(a: u64, b: u64) -> u64 {
    let r = (a as u128).wrapping_mul(b as u128);

    (r as u64) ^ (r >> 64) as u64
}

const RH1: u64 = 0x2d358dccaa6c78a5;
const RH2: u64 = 0x8bb84b93962eacc9;
const RH3: u64 = 0x4b33a62ed433d4a3;
// const RH4: u64 = 0x4d5a2da51de1aa47;
// const RH5: u64 = 0xa0761d6478bd642f;
// const RH6: u64 = 0xe7037ed1a0b428db;
// const RH7: u64 = 0x90ed1765281c388c;
const RH8: u64 = 0xaaaaaaaaaaaaaaaa;

///// rapidhash when the size of the input is known at compile time. This should ensure branches are compiled out.
//#[inline(always)]
//pub const fn rapidhash_known<const SIZE: usize>(seed: u64, bytes: &[u8; SIZE]) -> u64 {
//    rapidhash_known_core::<SIZE>(DEFAULT_SEED, bytes)
//}


//#[inline(always)]
//const fn rapidhash_known_core<const SIZE: usize>(mut seed: u64, bytes: &[u8]) -> u64 {
//    let a;
//    let b;
//
//    if SIZE <= 16 && SIZE >= 4 {
//        seed ^= SIZE as u64;
//    }
//
//    match SIZE {
//        0 => return finish(0, 0, seed, SIZE),
//        1..=3 => {
//            a = ((bytes[0] as u64) << 45) | bytes[SIZE - 1] as u64;
//            b = bytes[SIZE >> 1] as u64;
//        }
//        4..=8 => {
//            b = read_u32(bytes, 0) as u64;
//            a = read_u32(bytes, SIZE - 4) as u64;
//        }
//        9..=16 => {
//            a = read_u64(bytes, 0);
//            b = read_u64(bytes, SIZE - 8);
//        }
//        _ => return rapidhash_nano_core(seed, bytes), // this could also get compiled down to 0 branches with known size but its unlikely to matter most of the time.
//    }
//
//    finish(a, b, seed, SIZE)
//}