use crate::models::{ChangeHash, Identifier, TimestampInSeconds};
use core::str::from_utf8;
use minicbor::bytes::ByteVec;
use minicbor::encode::Write;
use minicbor::{Decode, Decoder, Encode, Encoder};
use ockam_core::compat::{collections::BTreeMap, vec::Vec};
use ockam_vault::{ECDSASHA256CurveP256Signature, EdDSACurve25519Signature};

/// Credential
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode)]
#[rustfmt::skip]
#[cbor(map)]
pub struct Credential {
    /// CBOR serialized [`super::VersionedData`]
    /// where VersionedData::data is CBOR serialized [`CredentialData`]
    #[cbor(with = "minicbor::bytes")]
    #[n(1)] pub data: Vec<u8>,
    /// Signature over data field using corresponding Credentials [`super::PurposeKeyAttestation`]
    #[n(2)] pub signature: CredentialSignature,
}

/// Signature over [`CredentialData`] using corresponding Credentials [`super::PurposeKeyAttestation`]
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode)]
#[rustfmt::skip]
pub enum CredentialSignature {
    /// An EdDSA signature using Curve 25519.
    #[n(1)] EdDSACurve25519(#[n(0)] EdDSACurve25519Signature),
    /// An ECDSA signature using SHA-256 and Curve P-256.
    #[n(2)] ECDSASHA256CurveP256(#[n(0)] ECDSASHA256CurveP256Signature),
}

/// Data inside a [`Credential`]
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode)]
#[rustfmt::skip]
#[cbor(map)]
pub struct CredentialData {
    /// To whom this Credential was issued
    #[n(1)] pub subject: Option<Identifier>,
    /// Latest Subject's Identity [`ChangeHash`] that was known to the Authority (issuer) at the
    /// moment of issuing of that Credential
    #[n(2)] pub subject_latest_change_hash: Option<ChangeHash>,
    /// [`Attributes`] that Authority (issuer) attests about that Subject
    #[n(3)] pub subject_attributes: Attributes,
    /// Creation [`TimestampInSeconds`] (UTC)
    #[n(4)] pub created_at: TimestampInSeconds,
    /// Expiration [`TimestampInSeconds`] (UTC)
    #[n(5)] pub expires_at: TimestampInSeconds,
}

/// Number that determines which keys&values to expect in the [`Attributes`]
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode)]
#[rustfmt::skip]
#[cbor(transparent)]
pub struct CredentialSchemaIdentifier(#[n(0)] pub u64);

/// Set a keys&values that an Authority (issuer) attests about the Subject
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode)]
#[rustfmt::skip]
#[cbor(map)]
pub struct Attributes {
    /// [`CredentialSchemaIdentifier`] that determines which keys&values to expect in the [`Attributes`]
    #[n(1)] pub schema: CredentialSchemaIdentifier,
    /// Set of keys&values
    /// The values exchanged with the Controller are ByteVec for attribute names so we need to adapt
    /// the encoding and decoding
    #[cbor(decode_with = "decode_attributes")]
    #[cbor(encode_with = "encode_attributes")]
    #[n(2)] pub map: BTreeMap<String, ByteVec>,
}

fn decode_attributes<'b, Ctx>(
    d: &mut Decoder<'b>,
    _ctx: &mut Ctx,
) -> Result<BTreeMap<String, ByteVec>, minicbor::decode::Error> {
    let attributes: BTreeMap<ByteVec, ByteVec> = d.decode()?;
    Ok(attributes
        .iter()
        .map(|(k, v)| {
            (
                from_utf8(k.as_slice())
                    .unwrap_or("expected a string")
                    .to_string(),
                v.to_owned(),
            )
        })
        .collect())
}

fn encode_attributes<Ctx, W: Write>(
    v: &BTreeMap<String, ByteVec>,
    e: &mut Encoder<W>,
    _ctx: &mut Ctx,
) -> Result<(), minicbor::encode::Error<W::Error>> {
    let attributes: BTreeMap<ByteVec, ByteVec> = v
        .iter()
        .map(|(k, v)| (From::from(k.as_bytes().to_vec()), v.to_owned()))
        .collect();
    e.encode(attributes)?;
    Ok(())
}
