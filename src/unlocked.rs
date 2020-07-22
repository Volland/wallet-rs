use crate::{
    contents::{
        key::{Key, KeyType},
        Content,
    },
    locked::LockedWallet,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ursa::{
    encryption::symm::prelude::*,
    hash::{sha3::Sha3_256, Digest},
};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct UnlockedWallet {
    pub context: Vec<String>,
    pub id: String,
    pub wallet_type: Vec<String>,
    contents: HashMap<String, Content>,
}

pub impl UnlockedWallet {
    pub fn sign_raw(&self, data: &[u8], key_ref: &str) -> Result<Vec<u8>, 'str> {
        match self.get_content(key_ref) {
            Some(c) => match c {
                Content::Key(k) => k.sign(data),
                _ => Err("incorrect content type".to_string()),
            },
            None => Err("no key found".to_string()),
        }
    }
    pub fn verify_raw(&self, data: &[u8], key_ref: &str, signature: &[u8]) -> Result<bool, String> {
        match self.contents.get(key_ref) {
            Some(c) => match c {
                Content::Key(k) => k.verify(data, signature),
                _ => Err("incorrect content type".to_string()),
            },
            None => Err("no key found".to_string()),
        }
    }
    pub fn decrypt(&self, data: &[u8], key_ref: &str) -> Result<Vec<u8>, String> {
        match self.contents.get(key_ref) {
            Some(c) => match c {
                Content::Key(k) => k.decrypt(data),
                _ => Err("incorrect content type".to_string()),
            },
            None => Err("no key found".to_string()),
        }
    }
    pub fn lock(&self, key: &[u8]) -> Result<LockedWallet, String> {
        let mut sha3 = Sha3_256::new();
        sha3.input(key);
        let pass = sha3.result();

        let aes = SymmetricEncryptor::<Aes256Gcm>::default();

        Ok(LockedWallet {
            encrypted_data: aes.encrypt_easy(self.id, self).map_err(|e| e.to_string())?,
        })
    }
}