// -*- mode: rust; -*-
//
// This file is part of schnorr-dalek.
// Copyright (c) 2018-2019 Web 3 Foundation
// See LICENSE for licensing information.
//
// Authors:
// - Jeff Burdges <jeff@web3.foundation>

//! Elliptic curve utilities not provided by curve25519-dalek,
//! including some not so safe utilities for managing scalars and points.

use curve25519_dalek::digest::{ExtendableOutput,XofReader};
use curve25519_dalek::edwards::EdwardsPoint;
use curve25519_dalek::ristretto::RistrettoPoint;
use curve25519_dalek::scalar::Scalar;

use errors::SignatureError;

pub fn scalar_from_xof<D>(hash: D) -> Scalar
where D: ExtendableOutput
{
    let mut output = [0u8; 64];
    hash.xof_result().read(&mut output);
    Scalar::from_bytes_mod_order_wide(&output)
}


/// Requires `RistrettoPoint` be defined as RistrettoPoint(EdwardsPoint)
pub fn ristretto_to_edwards(p: RistrettoPoint) -> EdwardsPoint {
    unsafe { ::std::mem::transmute::<RistrettoPoint,EdwardsPoint>(p) }
}

/// Requires `RistrettoPoint` be defined as RistrettoPoint(EdwardsPoint)
///
/// Avoid using this function.  It is necessarily painfully slow.
pub fn edwards_to_ristretto(p: EdwardsPoint) -> Result<RistrettoPoint,SignatureError> {
    if ! p.is_torsion_free() {
        return Err(SignatureError::PointDecompressionError);
    }
    Ok(unsafe { ::std::mem::transmute::<EdwardsPoint,RistrettoPoint>(p) })
}


pub fn divide_scalar_bytes_by_cofactor(scalar: &mut [u8; 32]) {
    let mut low = 0u8;
    for i in scalar.iter_mut().rev() {
        let r = *i & 0b00000111; // save remainder
        *i >>= 3; // divide by 8
        *i += low;
        low = r << 5;
    }
}

pub fn multiply_scalar_bytes_by_cofactor(scalar: &mut [u8; 32]) {
    let mut high = 0u8;
    for i in scalar.iter_mut() {
        let r = *i & 0b11100000; // carry bits
        *i <<= 3; // multiply by 8
        *i += high;
        high = r >> 5;
    }
}

pub fn divide_scalar_by_cofactor(scalar: Scalar) -> Scalar {
    let mut x = scalar.to_bytes();
    divide_scalar_bytes_by_cofactor(&mut x);
    Scalar::from_bits(x)
}

pub fn multiply_scalar_by_cofactor(scalar: Scalar) -> Scalar {
    let mut x = scalar.to_bytes();
    multiply_scalar_bytes_by_cofactor(&mut x);
    Scalar::from_bits(x)
}

#[cfg(test)]
mod tests {
    use super::*;
    // use ed25519_dalek::SecretKey;
    use rand::{thread_rng, Rng};

    // TODO: Simple test `RistrettoPoint` is implemented as an `EdwardsPoint`
    // #[test]
    // fn ristretto_point_is_edwards_point() {
    // }

    #[test]
    fn cofactor_adjustment() {
        let mut x: [u8; 32] = thread_rng().gen();
        x[31] &= 0b00011111;
        let mut y = x.clone();
        multiply_scalar_bytes_by_cofactor(&mut y);
        divide_scalar_bytes_by_cofactor(&mut y);
        assert_eq!(x, y);

        let mut x: [u8; 32] = thread_rng().gen();
        x[0] &= 0b11111000;
        let mut y = x.clone();
        divide_scalar_bytes_by_cofactor(&mut y);
        multiply_scalar_bytes_by_cofactor(&mut y);
        assert_eq!(x, y);
    }
}
