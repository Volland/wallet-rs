use super::encryption::unseal_box;
use super::public_key_info::{KeyType, PublicKeyInfo};
use secp256k1::{Message, Secp256k1, SecretKey};
use serde::{Deserialize, Serialize};
use ursa::{
    encryption::symm::prelude::*,
    kex::x25519::X25519Sha256,
    kex::KeyExchangeScheme,
    keys::{KeyGenOption, PrivateKey},
    signatures::prelude::*,
};
use crate::Error;
use sha3::{Digest, Keccak256};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KeyPair {
    #[serde(flatten)]
    pub public_key: PublicKeyInfo,
    pub private_key: PrivateKey,
}

impl KeyPair {
    pub fn new(key_type: KeyType, priv_key: &Vec<u8>) -> Result<KeyPair, Error> {
        let (pk, sk) = match key_type {
            KeyType::Ed25519VerificationKey2018 => {
                Ed25519Sha512::expand_keypair(&priv_key).map_err(|e| Error::UrsaCryptoError(e))?
            }
            KeyType::EcdsaSecp256k1VerificationKey2019
            | KeyType::EcdsaSecp256k1RecoveryMethod2020 => EcdsaSecp256k1Sha256::new()
                .keypair(Some(KeyGenOption::FromSecretKey(PrivateKey(
                    priv_key.clone(),
                ))))
                .map_err(|e| Error::UrsaCryptoError(e))?,
            KeyType::X25519KeyAgreementKey2019 => X25519Sha256::new()
                .keypair(Some(KeyGenOption::FromSecretKey(PrivateKey(
                    priv_key.clone(),
                ))))
                .map_err(|e| Error::UrsaCryptoError(e))?,
            _ => return Err(Error::UnsupportedKeyType),
        };

        Ok(KeyPair {
            public_key: PublicKeyInfo {
                controller: vec![],
                key_type: key_type,
                public_key: pk,
            },
            private_key: sk,
        })
    }

    pub fn random_pair(key_type: KeyType) -> Result<KeyPair, Error> {
        let (pk, sk) = match key_type {
            KeyType::X25519KeyAgreementKey2019 => {
                let x = X25519Sha256::new();
                x.keypair(None).map_err(|e| Error::UrsaCryptoError(e))?
            }
            KeyType::Ed25519VerificationKey2018 => {
                let ed = Ed25519Sha512::new();
                ed.keypair(None).map_err(|e| Error::UrsaCryptoError(e))?
            }
            KeyType::EcdsaSecp256k1VerificationKey2019
            | KeyType::EcdsaSecp256k1RecoveryMethod2020 => {
                let scp = EcdsaSecp256k1Sha256::new();
                scp.keypair(None).map_err(|e| Error::UrsaCryptoError(e))?
            }
            _ => return Err(Error::UnsupportedKeyType),
        };

        Ok(KeyPair {
            public_key: PublicKeyInfo {
                controller: vec![],
                key_type: key_type,
                public_key: pk,
            },
            private_key: sk,
        })
    }
    pub fn controller(self, controller: Vec<String>) -> Self {
        KeyPair {
            public_key: self.public_key.controller(controller),
            ..self
        }
    }
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        match self.public_key.key_type {
            KeyType::Ed25519VerificationKey2018 => {
                let ed = Ed25519Sha512::new();
                ed.sign(data, &self.private_key)
                    .map_err(|e| Error::UrsaCryptoError(e))
            }
            KeyType::EcdsaSecp256k1VerificationKey2019 => {
                let scp = EcdsaSecp256k1Sha256::new();
                scp.sign(data, &self.private_key)
                    .map_err(|e| Error::UrsaCryptoError(e))
            }
            KeyType::EcdsaSecp256k1RecoveryMethod2020 => {
                let scp = Secp256k1::new();
                let secp_secret_key = SecretKey::from_slice(&self.private_key.0)
                    .map_err(|e| Error::SecpCryptoError(e))?;

                let mut hasher = Keccak256::new();
                hasher.update(data);
                let output = hasher.finalize();

                let message = Message::from_slice(&output)
                    .map_err(|e| Error::SecpCryptoError(e))?;

                let sig = scp.sign_recoverable(&message, &secp_secret_key);
                let (rec_id, rs) = sig.serialize_compact();

                let rec_bit = rec_id.to_i32() as u8;

                let mut ret = rs.to_vec();
                ret.push(rec_bit);

                Ok(ret)
            }
            _ => Err(Error::WrongKeyType),
        }
    }
    pub fn decrypt(&self, data: &[u8], _aad: &[u8]) -> Result<Vec<u8>, Error> {
        match self.public_key.key_type {
            // default use xChaCha20Poly1905 with x25519 key agreement
            KeyType::X25519KeyAgreementKey2019 => unseal_box::<X25519Sha256, XChaCha20Poly1305>(
                data,
                &self.public_key.public_key,
                &self.private_key,
            ),
            _ => Err(Error::WrongKeyType),
        }
    }
    pub fn clean(&self) -> PublicKeyInfo {
        self.public_key.clone()
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PrivateKeyEncoding {
    // PrivateKeyJwk,
    PrivateKeyHex(String),
    PrivateKeyBase64(String),
    PrivateKeyBase58(String),
    PrivateKeyMultibase(String),
    PrivateKeyWebKms(String),
    PrivateKeySecureEnclave(String),
    PrivateKeyFromSeed { path: String, seed_ref: String },
}

#[test]
fn key_pair_new_ed25519() {
    // Test vector from https://fossies.org/linux/tor/src/test/ed25519_vectors.inc
    let test_sk =
        hex::decode("26c76712d89d906e6672dafa614c42e5cb1caac8c6568e4d2493087db51f0d36").unwrap();
    let expected_pk =
        hex::decode("c2247870536a192d142d056abefca68d6193158e7c1a59c1654c954eccaff894").unwrap();

    let key_entry = KeyPair::new(KeyType::Ed25519VerificationKey2018, &test_sk).unwrap();

    assert!(key_entry.public_key.key_type == KeyType::Ed25519VerificationKey2018);
    assert_eq!(key_entry.public_key.controller, Vec::<String>::new());
    assert_eq!(key_entry.public_key.public_key.0, expected_pk);
    assert_eq!(
        key_entry.private_key.0,
        [&test_sk[..], &expected_pk[..]].concat()
    )
}

#[test]
fn keccak256_correct_output() {
    let input = "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
    let mut hasher = Keccak256::new();
    hasher.update(input.as_bytes());
    let output = hasher.finalize();
    assert_eq!(hex::encode(output), "45d3b367a6904e6e8d502ee04999a7c27647f91fa845d456525fd352ae3d7371");
}

#[test]
fn key_pair_new_ecdsa_secp256k1() {
    // Self generated test vector.
    let test_sk =
        hex::decode("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855").unwrap();
    let expected_pk =
        hex::decode("03a34b99f22c790c4e36b2b3c2c35a36db06226e41c692fc82b8b56ac1c540c5bd").unwrap();

    let key_entry = KeyPair::new(KeyType::EcdsaSecp256k1VerificationKey2019, &test_sk).unwrap();

    assert!(key_entry.public_key.key_type == KeyType::EcdsaSecp256k1VerificationKey2019);
    assert_eq!(key_entry.public_key.controller, Vec::<String>::new());
    assert_eq!(key_entry.private_key.0, test_sk);
    assert_eq!(key_entry.public_key.public_key.0, expected_pk);
}

#[test]
fn key_pair_new_ecdsa_x25519() {
    // Test vector from https://tools.ietf.org/html/rfc7748#section-6.1
    let test_sk =
        hex::decode("a8abababababababababababababababababababababababababababababab6b").unwrap();
    let expected_pk =
        hex::decode("e3712d851a0e5d79b831c5e34ab22b41a198171de209b8b8faca23a11c624859").unwrap();

    let key_entry = KeyPair::new(KeyType::X25519KeyAgreementKey2019, &test_sk)?;

    assert!(key_entry.public_key.key_type == KeyType::X25519KeyAgreementKey2019);
    assert_eq!(key_entry.public_key.controller, Vec::<String>::new());
    assert_eq!(key_entry.private_key.0, test_sk);
    assert_eq!(key_entry.public_key.public_key.0, expected_pk);
    Ok(())
}
