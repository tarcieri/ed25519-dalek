// -*- mode: rust; -*-
//
// To the extent possible under law, the authors have waived all copyright and
// related or neighboring rights to curve25519-dalek, using the Creative
// Commons "CC0" public domain dedication.  See
// <http://creativecommons.org/publicdomain/zero/.0/> for full details.
//
// Authors:
// - Isis Agora Lovecruft <isis@patternsinthevoid.net>

//! A Rust implementation of ed25519 key generation, signing, and verification.

use core::fmt::Debug;

use sha2::{Digest, Sha512};

#[cfg(feature = "std")]
use rand::Rng;

use curve25519_dalek::curve;
use curve25519_dalek::curve::CompressedEdwardsY;
use curve25519_dalek::curve::ExtendedPoint;
use curve25519_dalek::curve::ProjectivePoint;
use curve25519_dalek::scalar::Scalar;
use curve25519_dalek::util::arrays_equal_ct;

pub const SIGNATURE_LENGTH: usize = 64;

/// An ed25519 signature.
///
/// # Note
///
/// These signatures, unlike the ed25519 reference implementation, are
/// "detached"—that is, they do **not** include a copy of the message which
/// has been signed.
#[derive(Copy)]
pub struct Signature(pub [u8; SIGNATURE_LENGTH]);

impl Clone for Signature {
    fn clone(&self) -> Self { *self }
}

impl Debug for Signature {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "Signature: {:?}", &self.0[..])
    }
}

impl Eq for Signature {}

impl PartialEq for Signature {
    /// # Note
    ///
    /// This function happens to be constant time, even though that is not
    /// really necessary.
    fn eq(&self, other: &Signature) -> bool {
        let mut equal: u8 = 0;

        for i in 0..64 {
            equal |= self.0[i] ^ other.0[i];
        }

        if equal == 0 {
            return true;
        } else {
            return false;
        }
    }
}

impl Signature {
    /// View this signature as an array of 64 bytes.
    #[inline]
    pub fn to_bytes(&self) -> [u8; SIGNATURE_LENGTH] {
        self.0
    }

    /// Construct a `Signature` from a slice of bytes.
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Signature {
        Signature(*array_ref!(bytes, 0, SIGNATURE_LENGTH))
    }
}

/// An ed25519 private key.
pub struct SecretKey(pub [u8; 64]);

impl Debug for SecretKey {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "SecretKey: {:?}", &self.0[..])
    }
}

impl SecretKey {
    /// View this secret key as an array of 32 bytes.
    #[inline]
    pub fn to_bytes(&self) -> [u8; 64] {
        self.0
    }

    /// Construct a `SecretKey` from a slice of bytes.
    ///
    /// # Warning
    ///
    /// **The caller is responsible for ensuring that the bytes represent a
    /// *masked* secret key.  If you do not understand what this means, DO NOT
    /// USE THIS CONSTRUCTOR.**
    ///
    /// # Example
    ///
    /// ```ignore
    /// use ed25519_dalek::SecretKey;
    ///
    /// let secret_key_bytes: [u8; 64] = [
    ///    157,  97, 177, 157, 239, 253,  90,  96, 186, 132,  74, 244, 146, 236,  44, 196,
    ///     68,  73, 197, 105, 123,  50, 105,  25, 112,  59, 172,   3,  28, 174, 127,  96,
    ///    215,  90, 152,   1, 130, 177,  10, 183, 213,  75, 254, 211, 201, 100,   7,  58,
    ///     14, 225, 114, 243, 218, 166,  35,  37, 175,   2,  26, 104, 247,   7,  81,  26];
    /// let public_key_bytes: [u8; 32] = [
    ///    215,  90, 152,   1, 130, 177,  10, 183, 213,  75, 254, 211, 201, 100,   7,  58,
    ///     14, 225, 114, 243, 218, 166,  35,  37, 175,   2,  26, 104, 247,   7,   81, 26];
    ///
    /// let secret_key: SecretKey = SecretKey::from_bytes(&[&secret_key_bytes[..32],
    ///                                                     &public_key_bytes[..32]].concat()[..]);
    /// ```
    ///
    /// # Returns
    ///
    /// A `SecretKey`.
    #[inline]
    #[allow(dead_code)]
    fn from_bytes(bytes: &[u8]) -> SecretKey {
        SecretKey(*array_ref!(bytes, 0, 64))
    }

    /// Sign a message with this keypair's secret key.
    pub fn sign(&self, message: &[u8]) -> Signature {
        let mut h: Sha512 = Sha512::new();
        let mut hash: [u8; 64] = [0u8; 64];
        let mut signature_bytes: [u8; 64] = [0u8; SIGNATURE_LENGTH];
        let mut expanded_key_secret: Scalar;
        let mesg_digest: Scalar;
        let hram_digest: Scalar;
        let r: ExtendedPoint;
        let s: Scalar;
        let t: CompressedEdwardsY;

        let secret_key: &[u8; 32] = array_ref!(&self.0,  0, 32);
        let public_key: &[u8; 32] = array_ref!(&self.0, 32, 32);

        h.input(secret_key);
        hash.copy_from_slice(h.result().as_slice());

        expanded_key_secret = Scalar(*array_ref!(&hash, 0, 32));
        expanded_key_secret[0]  &= 248;
        expanded_key_secret[31] &=  63;
        expanded_key_secret[31] |=  64;

        h = Sha512::new();
        h.input(&hash[32..]);
        h.input(&message);
        hash.copy_from_slice(h.result().as_slice());

        mesg_digest = Scalar::reduce(&hash);

        r = ExtendedPoint::basepoint_mult(&mesg_digest);

        h = Sha512::new();
        h.input(&r.compress().to_bytes()[..]);
        h.input(public_key);
        h.input(&message);
        hash.copy_from_slice(h.result().as_slice());

        hram_digest = Scalar::reduce(&hash);

        s = Scalar::multiply_add(&hram_digest, &expanded_key_secret, &mesg_digest);
        t = r.compress();

        signature_bytes[..32].copy_from_slice(&t.0);
        signature_bytes[32..64].copy_from_slice(&s.0);
        Signature(*array_ref!(&signature_bytes, 0, 64))
    }
}

/// An ed25519 public key.
#[derive(Copy, Clone)]
pub struct PublicKey(pub CompressedEdwardsY);

impl Debug for PublicKey {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "PublicKey( CompressedPoint( {:?} ))", self.0)
    }
}

impl PublicKey {
    /// View this public key as an array of 32 bytes.
    #[inline]
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    /// Construct a `PublicKey` from a slice of bytes.
    ///
    /// # Warning
    ///
    /// The caller is responsible for ensuring that the bytes passed into this
    /// method actually represent a `curve25519_dalek::curve::CompressedEdwardsY`
    /// and that said compressed point is actually a point on the curve.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use ed25519_dalek::PublicKey;
    ///
    /// let public_key_bytes: [u8; 32] = [
    ///    215,  90, 152,   1, 130, 177,  10, 183, 213,  75, 254, 211, 201, 100,   7,  58,
    ///     14, 225, 114, 243, 218, 166,  35,  37, 175,   2,  26, 104, 247,   7,   81, 26];
    ///
    /// let public_key: PublicKey = PublicKey::from_bytes(&public_key_bytes);
    ///
    /// ```
    ///
    /// # Returns
    ///
    /// A `PublicKey`.
    #[inline]
    #[allow(dead_code)]
    fn from_bytes(bytes: &[u8]) -> PublicKey {
        PublicKey(CompressedEdwardsY(*array_ref!(bytes, 0, 32)))
    }

    /// Convert this public key to its underlying extended twisted Edwards coordinate.
    #[inline]
    fn decompress(&self) -> Option<ExtendedPoint> {
        self.0.decompress()
    }

    /// Verify a signature on a message with this keypair's public key.
    ///
    /// # Return
    ///
    /// Returns true if the signature was successfully verified, and
    /// false otherwise.
    pub fn verify(&self, message: &[u8], signature: &Signature) -> bool {
        let mut h: Sha512 = Sha512::new();
        let mut a: ExtendedPoint;
        let ao:  Option<ExtendedPoint>;
        let r: ProjectivePoint;
        let mut digest: [u8; 64];
        let digest_reduced: Scalar;

        if signature.0[63] & 224 != 0 {
            return false;
        }
        ao = self.decompress();

        if ao.is_some() {
            a = ao.unwrap();
        } else {
            return false;
        }
        a = -(&a);

        digest = [0u8; 64];

        let top_half:    &[u8; 32] = array_ref!(&signature.0, 32, 32);
        let bottom_half: &[u8; 32] = array_ref!(&signature.0,  0, 32);

        h.input(&bottom_half[..]);
        h.input(&self.to_bytes());
        h.input(&message);
        digest.copy_from_slice(h.result().as_slice());

        digest_reduced = Scalar::reduce(&digest);
        r = curve::double_scalar_mult_vartime(&digest_reduced, &a, &Scalar(*top_half));

        if arrays_equal_ct(bottom_half, &r.compress().to_bytes()) == 1 {
            return true
        } else {
            return false
        }
    }
}

/// An ed25519 keypair.
#[derive(Debug)]
pub struct Keypair {
    /// The public half of this keypair.
    pub public: PublicKey,
    /// The secret half of this keypair.
    pub secret: SecretKey,
}

impl Keypair {
    /// Generate an ed25519 keypair.
    ///
    /// # Input
    ///
    /// A CSPRING with a `fill_bytes()` method, e.g. the one returned
    /// from `rand::OsRng::new()` (in the `rand` crate).
    // we reassign 0 bytes to the temp variable t to overwrite it
    #[cfg(feature = "std")]
    #[allow(unused_assignments)]
    pub fn generate<T: Rng>(cspring: &mut T) -> Keypair {
        let mut h: Sha512 = Sha512::new();
        let mut hash: [u8; 64] = [0u8; 64];
        let mut t:    [u8; 32] = [0u8; 32];
        let mut sk:   [u8; 64] = [0u8; 64];
        let     pk:   [u8; 32];
        let mut digest: &mut [u8; 32];

        cspring.fill_bytes(&mut t);

        h.input(&t);
        hash.copy_from_slice(h.result().as_slice());

        digest = array_mut_ref!(&mut hash, 0, 32);
        digest[0]  &= 248;
        digest[31] &= 127;
        digest[31] |= 64;

        pk = ExtendedPoint::basepoint_mult(&Scalar(*digest)).compress().to_bytes();

        for i in  0..32 {
            sk[i]    = t[i];
            sk[i+32] = pk[i];
            t[i]     = 0;
        }

        Keypair{
            public: PublicKey(CompressedEdwardsY(pk)),
            secret: SecretKey(sk),
        }
    }

    /// Sign a message with this keypair's secret key.
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.secret.sign(message)
    }

    /// Verify a signature on a message with this keypair's public key.
    pub fn verify(&self, message: &[u8], signature: &Signature) -> bool {
        self.public.verify(message, signature)
    }
}

#[cfg(test)]
mod test {
    use std::io::BufReader;
    use std::io::BufRead;
    use std::fs::File;
    use std::string::String;
    use std::vec::Vec;
    use test::Bencher;
    use curve25519_dalek::curve::ExtendedPoint;
    use rand::OsRng;
    use rand::Rng;
    use rustc_serialize::hex::FromHex;
    use super::*;

    /// A fake RNG which simply returns zeroes.
    struct ZeroRng;

    impl ZeroRng {
        fn new() -> ZeroRng {
            ZeroRng
        }
    }

    impl Rng for ZeroRng {
        fn next_u32(&mut self) -> u32 { 0u32 }

        fn fill_bytes(&mut self, bytes: &mut [u8]) {
            for i in 0 .. bytes.len() {
                bytes[i] = 0;
            }
        }
    }

    #[test]
    fn test_unmarshal_marshal() {  // TestUnmarshalMarshal
        let mut cspring: OsRng;
        let mut keypair: Keypair;
        let mut x: Option<ExtendedPoint>;
        let a: ExtendedPoint;
        let public: PublicKey;

        cspring = OsRng::new().unwrap();

        // from_bytes() fails if vx²-u=0 and vx²+u=0
        loop {
            keypair = Keypair::generate(&mut cspring);
            x = keypair.public.decompress();

            if x.is_some() {
                a = x.unwrap();
                break;
            }
        }
        public = PublicKey(a.compress());

        assert!(keypair.public.0 == public.0);
    }

    #[test]
    fn test_sign_verify() {  // TestSignVerify
        let mut cspring: OsRng;
        let keypair: Keypair;
        let good_sig: Signature;
        let bad_sig:  Signature;

        let good: &[u8] = "test message".as_bytes();
        let bad:  &[u8] = "wrong message".as_bytes();

        cspring  = OsRng::new().unwrap();
        keypair  = Keypair::generate(&mut cspring);
        good_sig = keypair.sign(&good);
        bad_sig  = keypair.sign(&bad);

        assert!(keypair.verify(&good, &good_sig) == true,
                "Verification of a valid signature failed!");
        assert!(keypair.verify(&good, &bad_sig)  == false,
                "Verification of a signature on a different message passed!");
        assert!(keypair.verify(&bad,  &good_sig) == false,
                "Verification of a signature on a different message passed!");
    }

    // TESTVECTORS is taken from sign.input.gz in agl's ed25519 Golang
    // package. It is a selection of test cases from
    // http://ed25519.cr.yp.to/python/sign.input
    #[cfg(test)]
    #[cfg(not(release))]
    #[test]
    fn test_golden() { // TestGolden
        let mut line: String;
        let mut lineno: usize = 0;

        let f = File::open("TESTVECTORS");
        if f.is_err() {
            println!("This test is only available when the code has been cloned \
                      from the git repository, since the TESTVECTORS file is large \
                      and is therefore not included within the distributed crate.");
            panic!();
        }
        let file = BufReader::new(f.unwrap());

        for l in file.lines() {
            lineno += 1;
            line = l.unwrap();

            let parts: Vec<&str> = line.split(':').collect();
            assert_eq!(parts.len(), 5, "wrong number of fields in line {}", lineno);

            let sec_bytes: &[u8] = &parts[0].from_hex().unwrap();
            let pub_bytes: &[u8] = &parts[1].from_hex().unwrap();
            let message:   &[u8] = &parts[2].from_hex().unwrap();
            let sig_bytes: &[u8] = &parts[3].from_hex().unwrap();

		    // The signatures in the test vectors also include the message
		    // at the end, but we just want R and S.
            let sig1: Signature = Signature::from_bytes(sig_bytes);

            assert_eq!(pub_bytes.len(), 32);

            let secret_key: SecretKey = SecretKey::from_bytes(&sec_bytes);
            let public_key: PublicKey = PublicKey::from_bytes(&pub_bytes);
            let sig2: Signature = secret_key.sign(&message);

            println!("{:?}", sec_bytes);
            println!("{:?}", pub_bytes);

            assert!(sig1 == sig2, "Signature bytes not equal on line {}", lineno);
            assert!(public_key.verify(&message, &sig2), "Signature verification failed on line {}", lineno);

        }
    }

    #[bench]
    fn bench_sign(b: &mut Bencher) {
        let mut cspring: OsRng = OsRng::new().unwrap();
        let keypair: Keypair = Keypair::generate(&mut cspring);
        let msg: &[u8] = "test message".as_bytes();

        b.iter(| | keypair.sign(msg));
    }

    #[bench]
    fn bench_verify(b: &mut Bencher) {
        let mut cspring: OsRng = OsRng::new().unwrap();
        let keypair: Keypair = Keypair::generate(&mut cspring);
        let msg: &[u8] = "test message".as_bytes();
        let sig: Signature = keypair.sign(msg);

        b.iter(| | keypair.verify(msg, &sig));
    }

    #[bench]
    fn bench_key_generation(b: &mut Bencher) {
        let mut rng: ZeroRng = ZeroRng::new();

        b.iter(| | Keypair::generate(&mut rng));
    }
}
