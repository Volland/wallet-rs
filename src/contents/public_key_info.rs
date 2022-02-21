use super::encryption::{KEYSIZE, seal_box};
use core::str::FromStr;
use std::convert::TryInto;
use crypto_box::PublicKey;
use serde::{Deserialize, Serialize};
use k256::ecdsa::{
    self,
    SigningKey,
    Signature,
    VerifyingKey,
    signature::Signer,
    recoverable
};
use crate::Error;

/// Holds public information on key, controller and type of the key.
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PublicKeyInfo {
    /// key controller information.
    pub controller: Vec<String>,
    /// variant of `KeyType` representing type of the key.
    #[serde(rename = "type")]
    pub key_type: KeyType,
    /// vector of bytes of public key.
    #[serde(rename = "publicKeyHex", with = "hex")]
    pub public_key: Vec<u8>,
}

impl PublicKeyInfo {
    /// Contstructor, which builds instance from `KeyType` and slice
    /// of bytes which are public key of type specified.
    // TODO: should this be checking if key matches len of declared type?
    ///
    /// # Parameters
    ///
    /// * kt - KeyType of provided key;
    /// * pk - public key as slice of bytes;
    ///
    /// # Examples
    /// ```
    /// # use crate::{
    /// #    universal_wallet::contents::public_key_info::{
    /// #               KeyType,
    /// #               PublicKeyInfo,
    /// #       },
    /// # };
    /// # fn test() {
    /// let pki = PublicKeyInfo::new(
    ///     KeyType::EcdsaSecp256k1VerificationKey2019,
    ///     &[0,0,0,0,0,0,0,0,0,0,0,0,0]);
    /// # }
    /// ```
    ///
    pub fn new(kt: KeyType, pk: &[u8]) -> Self {
        Self {
            controller: vec![],
            key_type: kt,
            public_key: pk.to_vec(),
        }
    }

    /// Sets controller property to provided value and returns updated struct.
    ///
    /// # Parameters
    ///
    /// * controller - `Vector` of `String`s to be set as new value.
    ///
    /// # Examples
    /// ```
    /// # use crate::{
    /// #    universal_wallet::contents::public_key_info::{
    /// #           KeyType,
    /// #           PublicKeyInfo,
    /// #       },
    /// # };
    /// # fn test() {
    /// let pki = PublicKeyInfo::new(KeyType::EcdsaSecp256k1VerificationKey2019,
    ///     &[0,0,0,0,0,0,0,0,0,0,0,0,0])
    ///     .controller(vec!("some new controller".into()));
    /// # }
    /// ```
    pub fn controller(self, controller: Vec<String>) -> Self {
        Self {
            controller: controller,
            ..self
        }
    }

    // TODO: should this cover all the key types?
    /// Encrypts message using own keys.
    ///
    /// # Parameters
    ///
    /// * data - message to be encrypted.
    /// * _aad - optional. not used ATM.
    ///
    /// # Examples
    /// ```
    /// # use crate::{
    /// #    universal_wallet::{
    /// #       contents::{
    /// #           public_key_info::{
    /// #               KeyType,
    /// #               PublicKeyInfo,
    /// #           },
    /// #           key_pair::KeyPair,
    /// #       },
    /// #   Error,
    /// #   }
    /// # };
    /// # fn test() -> Result<(), Error> {
    ///     let key_pair = KeyPair::random_pair(KeyType::X25519KeyAgreementKey2019)?;
    ///     let cipher_text = key_pair.public_key.encrypt(b"Super secret message", None)?;
    /// #   Ok(()) 
    /// # }
    pub fn encrypt(&self, data: &[u8], _aad: Option<&[u8]>) -> Result<Vec<u8>, Error> {
        match self.key_type {
            // default use xChaCha20Poly1905
            KeyType::X25519KeyAgreementKey2019 => {
                let pk: [u8; KEYSIZE] = self.public_key[..KEYSIZE]
                    .try_into()
                    .map_err(|_| Error::BoxToSmall)?;
                seal_box(data, &PublicKey::from(pk))
            }
            _ => Err(Error::WrongKeyType),
        }
    }

    /// Verifies validity of the signature provided.
    ///
    /// # Parameters
    ///
    /// * data - original message.
    /// * signature - generated by signing data.
    ///
    /// # Examples
    /// ```
    /// # use crate::{
    /// #    universal_wallet::{
    /// #       contents::{
    /// #           public_key_info::{
    /// #               KeyType,
    /// #               PublicKeyInfo,
    /// #           },
    /// #           key_pair::KeyPair,
    /// #       },
    /// #   Error,
    /// #   }
    /// # };
    /// # fn test() -> Result<(), Error> {
    ///     let key_pair = KeyPair::random_pair(KeyType::X25519KeyAgreementKey2019)?;
    ///     let signature = key_pair.sign(b"Not so secret stuff")?;
    ///     assert!(key_pair.public_key.verify(b"Not so secret stuff", &signature)?);
    /// #   Ok(()) 
    /// # }
    /// ```
    /// EcdsaSecp256k1 verification test:
    /// ```
    /// # use crate::{
    /// #    universal_wallet::{
    /// #       contents::{
    /// #           public_key_info::{
    /// #               KeyType,
    /// #               PublicKeyInfo,
    /// #           },
    /// #           key_pair::KeyPair,
    /// #       },
    /// #   Error,
    /// #   }
    /// # };
    /// # use std::str::FromStr;
    /// # fn test() -> Result<(), Error> {
    ///     let key = base64::decode_config("Aw2CKxqxbAH5CJK5fo0LqnREgJQYYsFcAocCKX7TrUmp",
    ///         base64::URL_SAFE);
    ///     let message = "hello there".as_bytes();
    ///     let signature = base64::decode_config(
    ///         "dxolMmEAt56BaIgqTdAZ17QmmNcOA9wkmiVNwtVLr_0Ob3r0R2v9lqDMQxF8Pt--Jl9BDDyaxIsYsbAybZv3rw==",
    ///         base64::URL_SAFE)?;
    ///#     let wrong_sig = base64::decode_config(
    ///#         "dxolAAAAt56BaIgqTdAZ17QmmNcOA9wkmiVNwtVLr_0Ob3r0R2v9lqDMQxF8Pt--Jl9BDDyaxIsYsbAybZv3rw==",
    ///#         base64::URL_SAFE)?;
    ///     let pki = PublicKeyInfo::new(KeyType::from_str("EcdsaSecp256k1VerificationKey2019")?, &key?);
    ///     assert!(pki.verify(message, &signature)?);
    ///#     assert!(!pki.verify(message, &wrong_sig)?);
    /// # Ok(())}
    /// ```
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, Error> {
        match self.key_type {
            KeyType::Ed25519VerificationKey2018 => {
                use ed25519_dalek::{PublicKey, Verifier, Signature};
                let pk = PublicKey::from_bytes(&self.public_key)
                    .map_err(|e| Error::Other(Box::new(e)))?;
                if signature.len() != 64 {
                    return Err(Error::WrongKeyLength);
                }
                let owned_signature = signature.to_owned();
                let array_signature = array_ref!(owned_signature, 0, 64).to_owned();
                let signature = Signature::from(array_signature);

                Ok(pk.verify(data, &signature).is_ok())
            },
            KeyType::EcdsaSecp256k1VerificationKey2019 => {
                let vk = VerifyingKey::from_sec1_bytes(&self.public_key)?;
                let s1: [u8; 32] = array_ref!(signature, 0, 32).to_owned();
                let s2: [u8; 32] = array_ref!(signature, 32, 32).to_owned();
                let sign = Signature::from_scalars(s1, s2)?;
                use k256::ecdsa::signature::Verifier;
                Ok(vk.verify(data, &sign).is_ok())
            },
            KeyType::EcdsaSecp256k1RecoveryMethod2020 => {
                let s1: [u8; 32] = array_ref!(signature, 0, 32).to_owned();
                let s2: [u8; 32] = array_ref!(signature, 32, 32).to_owned();
                let rs = ecdsa::Signature::from_scalars(s1, s2)
                    .map_err(|e| Error::EdCryptoError(e))?;
                let recovered_signature = recoverable::Signature::from_trial_recovery(
                    &ecdsa::VerifyingKey::from_sec1_bytes(&self.public_key)?,
                    data,
                    &rs
                ).map_err(|oe| Error::EcdsaCryptoError(oe))?;

                let recovered_key = recovered_signature.recover_verify_key(data)
                    .map_err(|e| Error::EcdsaCryptoError(e))?;

                let our_key = ecdsa::VerifyingKey::from_sec1_bytes(&self.public_key).map_err(|e| Error::EcdsaCryptoError(e))?;

                Ok(our_key == recovered_key)
            },
            KeyType::Bls12381G1Key2020 => {
                use signature_bls::{SignatureVt, PublicKeyVt};
                let pk = PublicKeyVt::from_bytes(array_ref!(&self.public_key, 0, 48)).unwrap();
                Ok(SignatureVt::from_bytes(array_ref!(signature, 0, 96)).unwrap().verify(pk, signature).unwrap_u8() == 1u8)
            },
            KeyType::Bls12381G2Key2020 => {
                use signature_bls::{Signature, PublicKey};
                let pk = PublicKey::from_bytes(array_ref!(&self.public_key, 0, 96)).unwrap();
                Ok(Signature::from_bytes(array_ref!(signature, 0, 48)).unwrap().verify(pk, signature).unwrap_u8() == 1u8)
            }
            _ => Err(Error::WrongKeyType),
        }
    }
}

/// Lists all supported* keys.
/// TODO: find links to all the key specs.
/// [W3C](https://w3c-ccg.github.io/ld-cryptosuite-registry/#jsonwebsignature2020)
/// `JwsVerificationKey2020`
/// [W3C](https://w3c-ccg.github.io/ld-cryptosuite-registry/#ecdsasecp256k1signature2019)
/// `EcdsaSecp256k1VerificationKey2019`
/// [W3C](https://w3c-ccg.github.io/ld-cryptosuite-registry/#ed25519)
/// `Ed25519VerificationKey2018`
/// [W3C](https://w3c-ccg.github.io/ld-cryptosuite-registry/#gpgsignature2020)
/// `GpgVerificationKey2020`
/// [W3C](https://w3c-ccg.github.io/ld-cryptosuite-registry/#rsasignature2018)
/// `RsaVerificationKey2018`
/// [W3C](https://w3c-ccg.github.io/ld-cryptosuite-registry/#ecdsasecp256k1recoverysignature2020)
/// `EcdsaSecp256k1RecoveryMethod2020`
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum KeyType {
    JwsVerificationKey2020,
    EcdsaSecp256k1VerificationKey2019,
    Ed25519VerificationKey2018,
    GpgVerificationKey2020,
    RsaVerificationKey2018,
    X25519KeyAgreementKey2019,
    SchnorrSecp256k1VerificationKey2019,
    EcdsaSecp256k1RecoveryMethod2020,
    Bls12381G1Key2020,
    Bls12381G2Key2020
}

impl FromStr for KeyType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "JwsVerificationKey2020" => Ok(Self::JwsVerificationKey2020),
            "EcdsaSecp256k1VerificationKey2019" => Ok(Self::EcdsaSecp256k1VerificationKey2019),
            "Ed25519VerificationKey2018" => Ok(Self::Ed25519VerificationKey2018),
            "GpgVerificationKey2020" => Ok(Self::GpgVerificationKey2020),
            "RsaVerificationKey2018" => Ok(Self::RsaVerificationKey2018),
            "X25519KeyAgreementKey2019" => Ok(Self::X25519KeyAgreementKey2019),
            "SchnorrSecp256k1VerificationKey2019" => Ok(Self::SchnorrSecp256k1VerificationKey2019),
            "EcdsaSecp256k1RecoveryMethod2020" => Ok(Self::EcdsaSecp256k1RecoveryMethod2020),
            "Bls12381G1Key2020" => Ok(Self::Bls12381G1Key2020),
            "Bls12381G2Key2020" => Ok(Self::Bls12381G2Key2020),
            _ => Err(Error::UnsupportedKeyType),
        }
    }
}

impl TryInto<KeyType> for &str {
    type Error = Error ;

    fn try_into(self) -> Result<KeyType, Error> {
        match self {
            "JwsVerificationKey2020" => Ok(KeyType::JwsVerificationKey2020),
            "EcdsaSecp256k1VerificationKey2019" => Ok(KeyType::EcdsaSecp256k1VerificationKey2019),
            "Ed25519VerificationKey2018" => Ok(KeyType::Ed25519VerificationKey2018),
            "GpgVerificationKey2020" => Ok(KeyType::GpgVerificationKey2020),
            "RsaVerificationKey2018" => Ok(KeyType::RsaVerificationKey2018),
            "X25519KeyAgreementKey2019" => Ok(KeyType::X25519KeyAgreementKey2019),
            "SchnorrSecp256k1VerificationKey2019" => Ok(KeyType::SchnorrSecp256k1VerificationKey2019),
            "EcdsaSecp256k1RecoveryMethod2020" => Ok(KeyType::EcdsaSecp256k1RecoveryMethod2020),
            "Bls12381G1Key2020" => Ok(KeyType::Bls12381G1Key2020),
            "Bls12381G2Key2020" => Ok(KeyType::Bls12381G2Key2020),
            _ => Err(Error::UnsupportedKeyType),
        }
    }
}

/// Defines encoding for public keys.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PublicKeyEncoding {
    // TODO, find a good JWK def crate
    // PublicKeyJwk,
    PublicKeyHex(String),
    PublicKeyBase64(String),
    PublicKeyBase58(String),
    PublicKeyMultibase(String),
    EthereumAddress(String),
}

// TODO: find out if they still required by any consumer
// cleanup if not...

pub fn to_recoverable_signature(
    _v: u8,
    r: &[u8; 32],
    s: &[u8; 32],
) -> Result<recoverable::Signature, Error> {
    let s_key = SigningKey::random(rand::rngs::OsRng);
    let mut data = [0u8; 64];
    data[0..32].copy_from_slice(r);
    data[32..64].copy_from_slice(s);

    Ok(s_key.sign(&data))
}

pub fn parse_concatenated(signature: &[u8]) -> Result<recoverable::Signature, Error> {
    let mut r = [0u8; 32];
    let mut s = [0u8; 32];
    let v = signature[64];

    r.copy_from_slice(&signature[..32]);
    s.copy_from_slice(&signature[32..64]);

    println!("{:?}", signature);
    println!("{:?}", r);
    println!("{:?}", s);
    println!("{:?}", v);

    to_recoverable_signature(v, &r, &s)
}

#[test]
fn key_type_from_str_test() -> Result<(), Error> {
    // Arrange + Act
    let kt = KeyType::from_str("EcdsaSecp256k1VerificationKey2019")?;
    let kt2: KeyType = "EcdsaSecp256k1VerificationKey2019".try_into()?;
    let expected = KeyType::EcdsaSecp256k1VerificationKey2019;
    println!("{:?}", &expected);
    println!("{:?}", kt);
    println!("{:?}", kt2);
    // Assert
    assert_eq!(expected, KeyType::EcdsaSecp256k1VerificationKey2019);
    assert_eq!(kt, kt2);
    Ok(())
}

#[test]
fn ecdsa_signature_test() -> Result<(), Error> {
    let key = base64::decode_config("Aw2CKxqxbAH5CJK5fo0LqnREgJQYYsFcAocCKX7TrUmp",
    base64::URL_SAFE);
    let message = "hello there".as_bytes();
    let signature = base64::decode_config(
    "dxolMmEAt56BaIgqTdAZ17QmmNcOA9wkmiVNwtVLr_0Ob3r0R2v9lqDMQxF8Pt--Jl9BDDyaxIsYsbAybZv3rw==",
    base64::URL_SAFE)?;
    let wrong_sig = base64::decode_config(
    "dxolAAAAt56BaIgqTdAZ17QmmNcOA9wkmiVNwtVLr_0Ob3r0R2v9lqDMQxF8Pt--Jl9BDDyaxIsYsbAybZv3rw==",
    base64::URL_SAFE)?;
    let pki = PublicKeyInfo::new(KeyType::from_str("EcdsaSecp256k1VerificationKey2019")?, &key?);
    assert!(pki.verify(message, &signature)?);
    assert!(!pki.verify(message, &wrong_sig)?);
    Ok(())
}

#[test]
fn ecdsa_private_public_keys_full_cycle_test() -> Result<(), Error> {
    // Arrange
    use crate::contents::key_pair::KeyPair;
    let pk = 
        hex::decode("ebb2c082fd7727890a28ac82f6bdf97bad8de9f5d7c9028692de1a255cad3e0f")
            .unwrap();
    let pub_key = 
        hex::decode("04779dd197a5df977ed2cf6cb31d82d43328b790dc6b3b7d4437a427bd5847dfcde94b724a555b6d017bb7607c3e3281daf5b1699d6ef4124975c9237b917d426f")
            .unwrap();
    let expected =
        hex::decode("46e830fcc9f6cee692752ab1fb7307db7a02f74b6e44fd92faffd3e9e45f5f7b3cadb5fddfc4a794edcd079c9772f48da21eff6590838661de0f21cb6bf5fdb1")
            .unwrap();
    let message =
        hex::decode("4b688df40bcedbe641ddb16ff0a1842d9c67ea1c3bf63f3e0471baa664531d1a")
            .unwrap();
    let pki = PublicKeyInfo::new(KeyType::EcdsaSecp256k1VerificationKey2019, &pub_key);
    let mut kp = KeyPair::new(KeyType::EcdsaSecp256k1VerificationKey2019, &pk)?;
    kp.public_key = pki;
    // Act
    let sign = kp.sign(&message)?;
    // Assert
    assert_eq!(&expected, &sign);
    assert!(&kp.public_key.verify(&message, &sign)?);
    Ok(())
}
