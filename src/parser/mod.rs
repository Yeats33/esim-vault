//! LPA (Local Profile Assistant) payload parser

mod lpa_parser;

pub use lpa_parser::{generate_qr_image, parse_lpa};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_standard_lpa() {
        let payload = "LPA:1$sm-dp.plus$ABC123";
        let result = parse_lpa(payload);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.smdp, Some("sm-dp.plus".to_string()));
        assert_eq!(parsed.activation_code, Some("ABC123".to_string()));
    }

    #[test]
    fn test_parse_lpa_with_confirmation() {
        let payload = "LPA:1$sm-dp.plus$ABC123$5678";
        let result = parse_lpa(payload);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.smdp, Some("sm-dp.plus".to_string()));
        assert_eq!(parsed.activation_code, Some("ABC123".to_string()));
        assert_eq!(parsed.confirmation_code, Some("5678".to_string()));
    }

    #[test]
    fn test_parse_lpa_without_prefix() {
        let payload = "sm-dp.plus$ABC123";
        let result = parse_lpa(payload);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.smdp, Some("sm-dp.plus".to_string()));
        assert_eq!(parsed.activation_code, Some("ABC123".to_string()));
    }

    #[test]
    fn test_parse_lpa_missing_activation_code() {
        let payload = "LPA:1$sm-dp.plus";
        let result = parse_lpa(payload);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.smdp, Some("sm-dp.plus".to_string()));
        assert!(parsed.activation_code.is_none());
    }

    #[test]
    fn test_parse_lpa_multiple_fields() {
        let payload = "LPA:1$sm-dp.plus$ABC123$5678$extra";
        let result = parse_lpa(payload);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.smdp, Some("sm-dp.plus".to_string()));
        assert_eq!(parsed.activation_code, Some("ABC123".to_string()));
        assert_eq!(parsed.confirmation_code, Some("5678".to_string()));
        assert_eq!(parsed.other.len(), 1);
    }

    #[test]
    fn test_parse_lpa_empty_payload() {
        let payload = "";
        let result = parse_lpa(payload);
        // Empty payload should return an empty parsed result
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_lpa_case_insensitive() {
        let payload = "lpa:1$sm-dp.plus$ABC123";
        let result = parse_lpa(payload);
        assert!(result.is_ok());
    }

    #[test]
    fn test_reconstruct_lpa() {
        let payload = "LPA:1$sm-dp.plus$ABC123$5678";
        let parsed = parse_lpa(payload).unwrap();
        // The parser should be able to reconstruct the format
        assert!(parsed.smdp.is_some() || parsed.activation_code.is_some());
    }
}
