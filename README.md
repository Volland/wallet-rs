# Wallet-rs
[![Build Status](https://img.shields.io/endpoint.svg?url=https%3A%2F%2Factions-badge.atrox.dev%2F35359595%2Fwallet-rs%2Fbadge%3Fref%3Dmaster&style=for-the-badge)](https://actions-badge.atrox.dev/35359595/wallet-rs/goto?ref=master)

Wallet-rs is a Rust implementation of the [Universal Wallet Specification](https://transmute-industries.github.io/universal-wallet), (currently) focusing on simplicity and correctness. Currently it implements a small subset of the described functionality, namely key generation, storage, operations (signing, key agreement, decryption) and export of public key material. It is currently intended and used for a secure key management solution for use in SSI agents in mobile devices, web services and browsers.
