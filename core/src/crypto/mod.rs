//! Crypto related functionality. It is used for establishing
//! trust between a client and server via certificate exchange and validation. It also used for
//! encrypting / decrypting messages and signing messages.

pub mod x509;
pub mod aeskey;
pub mod pkey;
pub mod thumbprint;
pub mod certificate_store;
pub mod hash;
pub mod security_policy;

pub use self::x509::*;
pub use self::aeskey::*;
pub use self::pkey::*;
pub use self::thumbprint::*;
pub use self::certificate_store::*;
pub use self::hash::*;
pub use self::security_policy::*;

use opcua_types::*;

// Size of a SHA1 hash value in bytes
pub const SHA1_SIZE: usize = 20;
// Size of a SHA256 hash value bytes
pub const SHA256_SIZE: usize = 32;

fn concat_data_and_nonce(data: &[u8], nonce: &[u8]) -> Vec<u8> {
    let mut buffer: Vec<u8> = Vec::with_capacity(data.len() + nonce.len());
    buffer.extend_from_slice(data);
    buffer.extend_from_slice(nonce);
    buffer
}

/// Verifies that cert matches the signed data
pub fn verify_signature(verifying_cert: &X509, signature_data: &SignatureData, data: &ByteString, nonce: &ByteString) -> Result<StatusCode, StatusCode> {
    if data.is_null() || nonce.is_null() {
        error!("Data or nonce are null");
        Err(BAD_UNEXPECTED_ERROR)
    } else if signature_data.algorithm.is_null() {
        error!("Signature data has no algorithm");
        Err(BAD_UNEXPECTED_ERROR)
    } else if let Ok(public_key) = verifying_cert.public_key() {
        // Get the public key
        let data = concat_data_and_nonce(data.as_ref(), nonce.as_ref());
        let signature = signature_data.signature.as_ref();

        let security_policy_uri = signature_data.algorithm.as_ref();
        let security_policy = SecurityPolicy::from_uri(security_policy_uri);

        let verified = match security_policy {
            SecurityPolicy::Basic128Rsa15 | SecurityPolicy::Basic256 => {
                public_key.verify_hmac_sha1(&data, signature)?
            }
            SecurityPolicy::Basic256Sha256 => {
                public_key.verify_hmac_sha256(&data, signature)?
            }
            SecurityPolicy::None => {
                error!("Cannot verify a signature with no security policy of None");
                false
            }
            _ => {
                error!("An unknown security policy uri {} was passed to signing function and rejected", security_policy_uri);
                false
            }
        };
        Ok(if verified { GOOD } else { BAD_APPLICATION_SIGNATURE_INVALID })
    } else {
        error!("Public key cannot be obtained from cert");
        Err(BAD_UNEXPECTED_ERROR)
    }
}

/// Creates a `SignatureData` object by signing the supplied certificate and nonce with a pkey
pub fn create_signature_data(pkey: &PKey, security_policy_uri: &str, data: &ByteString, nonce: &ByteString) -> Result<SignatureData, StatusCode> {
    let (algorithm, signature) = if data.is_null() || nonce.is_null() {
        (UAString::null(), ByteString::null())
    } else {
        let data = concat_data_and_nonce(data.as_ref(), nonce.as_ref());

        // Sign the bytes and return the algorithm, signature
        let security_policy = SecurityPolicy::from_uri(security_policy_uri);
        match security_policy {
            SecurityPolicy::Basic128Rsa15 | SecurityPolicy::Basic256 => {
                let mut signature = [0u8; SHA1_SIZE];
                let _ = pkey.sign_hmac_sha1(&data, &mut signature)?;
                (
                    UAString::from(security_policy.asymmetric_signature_algorithm()),
                    ByteString::from(&signature)
                )
            }
            SecurityPolicy::Basic256Sha256 => {
                let mut signature = [0u8; SHA256_SIZE];
                let _ = pkey.sign_hmac_sha256(&data, &mut signature)?;
                (
                    UAString::from(security_policy.asymmetric_signature_algorithm()),
                    ByteString::from(&signature)
                )
            }
            SecurityPolicy::None => (
                UAString::null(), ByteString::null()
            ),
            _ => {
                error!("An unknown security policy uri {} was passed to signing function and rejected", security_policy_uri);
                (UAString::null(), ByteString::null())
            }
        }
    };
    Ok(SignatureData { algorithm, signature })
}