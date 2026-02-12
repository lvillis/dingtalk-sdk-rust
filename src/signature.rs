use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::Error;
use crate::error::Result;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use hmac::{Hmac, Mac};
use sha2::Sha256;

pub(crate) fn current_timestamp_millis() -> Result<String> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis()
        .to_string())
}

fn sign_hmac_sha256(secret: &str, content: &str) -> Result<Vec<u8>> {
    let mut mac =
        Hmac::<Sha256>::new_from_slice(secret.as_bytes()).map_err(|_| Error::Signature)?;
    mac.update(content.as_bytes());
    Ok(mac.finalize().into_bytes().to_vec())
}

pub(crate) fn create_signature(timestamp: &str, secret: &str) -> Result<String> {
    let string_to_sign = format!("{timestamp}\n{secret}");
    let signature = sign_hmac_sha256(secret, &string_to_sign)?;
    let signature_base64 = STANDARD.encode(signature);
    Ok(urlencoding::encode(&signature_base64).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_expected_signature() {
        let signature = create_signature("1700000000000", "secret").expect("signature");
        assert_eq!(
            signature,
            "OuzzJR5%2BxZ4%2FEYwqtNt6sMYZQMTa%2FHEGvc9miJe7XzY%3D"
        );
    }
}
