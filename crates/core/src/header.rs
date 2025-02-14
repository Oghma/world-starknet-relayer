use core::fmt;
use std::ops;

use alloy_consensus::{Sealable, Sealed};
use alloy_primitives::{hex, keccak256, B256};
use alloy_rlp::{Decodable, Encodable};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

/// A wrapper type for headers that caches their RLP encoding.
///
/// This provides efficient serialization/deserialization while maintaining
/// the ability to compute the block hash using the cached RLP bytes.
///
/// Note: The code is taken from: https://github.com/risc0/risc0-ethereum/blob/main/crates/steel/src/ethereum.rs
#[derive(Clone, Debug)]
pub struct RlpHeader<H: Encodable> {
    /// The inner header type being wrapped
    inner: H,
    /// Cached RLP bytes for efficient hash computation
    rlp: Option<Box<[u8]>>,
}

impl<H: Encodable> ops::Deref for RlpHeader<H> {
    /// Provides direct access to the inner header's fields
    type Target = H;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<H: Encodable> RlpHeader<H> {
    /// Creates a new RlpHeader without precomputed RLP
    #[must_use]
    pub const fn new(inner: H) -> Self {
        Self { inner, rlp: None }
    }

    /// Returns a reference to the inner header
    pub fn inner(&self) -> &H {
        &self.inner
    }

    /// Returns a mutable reference to the inner header
    pub fn inner_mut(&mut self) -> &mut H {
        &mut self.inner
    }

    /// Consumes the wrapper, returning the inner header
    pub fn into_inner(self) -> H {
        self.inner
    }
}

impl<H: Encodable> Sealable for RlpHeader<H> {
    /// Computes the block hash by keccak256 of the RLP-encoded header
    ///
    /// Uses cached RLP if available, otherwise encodes the header
    #[inline]
    fn hash_slow(&self) -> B256 {
        match &self.rlp {
            Some(rlp) => keccak256(rlp),
            None => keccak256(alloy_rlp::encode(&self.inner)),
        }
    }

    /// Seals the header with a precomputed block hash
    ///
    /// Note: Resets the RLP cache as the seal may not match the current state
    #[inline]
    fn seal_unchecked(mut self, seal: B256) -> Sealed<Self> {
        self.rlp = None;
        Sealed::new_unchecked(self, seal)
    }
}

impl<H: Encodable> Serialize for RlpHeader<H> {
    /// Serializes the header using RLP encoding
    ///
    /// - Human-readable formats: hex-encoded RLP
    /// - Binary formats: raw RLP bytes
    #[inline]
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let encoded = alloy_rlp::encode(&self.inner);
        if serializer.is_human_readable() {
            hex::serialize(&encoded, serializer)
        } else {
            serializer.serialize_bytes(&encoded)
        }
    }
}

impl<'de, H: Encodable + Decodable> Deserialize<'de> for RlpHeader<H> {
    /// Deserializes from RLP-encoded data
    ///
    /// Supports both hex-encoded strings (human-readable) and raw bytes
    #[inline]
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let rlp = if deserializer.is_human_readable() {
            deserializer.deserialize_any(BytesVisitor)?
        } else {
            deserializer.deserialize_byte_buf(BytesVisitor)?
        };
        let inner = alloy_rlp::decode_exact(&rlp).map_err(de::Error::custom)?;

        Ok(RlpHeader {
            inner,
            rlp: Some(rlp.into_boxed_slice()),
        })
    }
}

struct BytesVisitor;

impl<'de> de::Visitor<'de> for BytesVisitor {
    type Value = Vec<u8>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("bytes represented as a hex string, sequence or raw bytes")
    }
    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        hex::decode(v).map_err(de::Error::custom)
    }
    fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Self::Value, E> {
        Ok(v.to_vec())
    }
    fn visit_byte_buf<E: de::Error>(self, v: Vec<u8>) -> Result<Self::Value, E> {
        Ok(v)
    }
    fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut values = Vec::with_capacity(seq.size_hint().unwrap_or(0));
        while let Some(value) = seq.next_element()? {
            values.push(value);
        }
        Ok(values)
    }
}
