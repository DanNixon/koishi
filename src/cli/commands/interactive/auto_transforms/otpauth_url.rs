use super::AutoTransform;
use zeroize::Zeroizing;

pub(super) struct OtpauthUrl {}

impl AutoTransform for OtpauthUrl {
    fn applies(data: &[u8]) -> miette::Result<bool> {
        let expected_prefix = b"otpauth://";
        match &data.get(0..expected_prefix.len()) {
            Some(data_prefix) => Ok(data_prefix == expected_prefix),
            None => Ok(false),
        }
    }

    fn apply(data: Zeroizing<Vec<u8>>) -> miette::Result<Zeroizing<Vec<u8>>> {
        let otp_key = crate::utils::bytes_to_string(data)?;
        let otp_pass = crate::utils::totp_from_otpauth(otp_key)?;
        Ok(otp_pass.as_bytes().to_vec().into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_with_otpauth_url() {
        let data = Zeroizing::new(
            b"otpauth://totp/Example:alice@google.com?secret=JBSWY3DPEHPK3PXP".to_vec(),
        );
        assert!(OtpauthUrl::applies(&data).unwrap());
    }

    #[test]
    fn applies_with_any_old_string() {
        let data = Zeroizing::new(b"the sky is blue".to_vec());
        assert!(!OtpauthUrl::applies(&data).unwrap());
    }

    #[test]
    fn applies_with_empty_string() {
        let data = Zeroizing::new(b"".to_vec());
        assert!(!OtpauthUrl::applies(&data).unwrap());
    }
}
