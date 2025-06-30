mod otpauth_url;

use zeroize::Zeroizing;

pub(super) fn process(data: Zeroizing<Vec<u8>>) -> miette::Result<Zeroizing<Vec<u8>>> {
    if otpauth_url::OtpauthUrl::applies(&data)? {
        otpauth_url::OtpauthUrl::apply(data)
    } else {
        Ok(data)
    }
}

trait AutoTransform {
    fn applies(data: &[u8]) -> miette::Result<bool>;
    fn apply(data: Zeroizing<Vec<u8>>) -> miette::Result<Zeroizing<Vec<u8>>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use zeroize::Zeroizing;

    #[test]
    fn test_process_otpauth_url() {
        let data = Zeroizing::new(
            b"otpauth://totp/Example:alice@google.com?secret=JBSWY3DPEHPK3PXP".to_vec(),
        );
        let result = process(data.clone()).unwrap();
        assert_eq!(result.len(), 6);
    }

    #[test]
    fn test_process_non_otpauth_url() {
        let data = Zeroizing::new(b"not an otpauth url".to_vec());
        let result = process(data.clone()).unwrap();
        assert_eq!(*result, *data);
    }
}
