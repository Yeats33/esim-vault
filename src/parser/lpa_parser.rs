//! LPA (Local Profile Assistant) payload parser implementation

use regex::Regex;

use crate::core::lpa::{LpaField, ParsedLpa};
use crate::error::{Error, Result};

/// Parse an LPA payload string into structured fields
/// 
/// Supported formats:
/// - `LPA:1$<smdp>$<activation_code>[$<confirmation_code>][$...]`
/// - `<smdp>$<activation_code>[$<confirmation_code>][$...]`
/// 
/// The LPA URI scheme is defined in GSMA SGP.22
pub fn parse_lpa(payload: &str) -> Result<ParsedLpa> {
    let payload = payload.trim();
    
    if payload.is_empty() {
        return Ok(ParsedLpa::default());
    }

    // Remove "LPA:" prefix if present (case-insensitive)
    let parts: Vec<&str> = if payload.to_uppercase().starts_with("LPA:") {
        // Find the first $ to split prefix from data
        if let Some(dollar_pos) = payload.find('$') {
            let prefix = &payload[..dollar_pos];
            let rest = &payload[dollar_pos + 1..];
            
            // Check if there's a version after LPA:
            let format = if prefix.len() > 4 {
                Some(prefix[4..].to_string())
            } else {
                Some("1".to_string())
            };
            
            let mut result = parse_delimited(rest, '$');
            result.format = format;
            return Ok(result);
        } else {
            // No $ found, try to parse as simple format
            payload[4..].split('$').collect()
        }
    } else {
        // No LPA: prefix, assume raw delimited format
        payload.split('$').collect()
    };

    Ok(parse_parts(&parts))
}

fn parse_delimited(data: &str, delimiter: char) -> ParsedLpa {
    let parts: Vec<&str> = data.split(delimiter).filter(|s| !s.is_empty()).collect();
    parse_parts(&parts)
}

fn parse_parts(parts: &[&str]) -> ParsedLpa {
    let mut parsed = ParsedLpa::default();
    
    // Field 0: SM-DP+ address
    if parts.len() > 0 {
        let smdp = parts[0].to_string();
        if !smdp.is_empty() {
            parsed.smdp = Some(smdp);
        }
    }
    
    // Field 1: Activation code
    if parts.len() > 1 {
        let ac = parts[1].to_string();
        if !ac.is_empty() {
            parsed.activation_code = Some(ac);
        }
    }
    
    // Field 2: Confirmation code (optional)
    if parts.len() > 2 {
        let cc = parts[2].to_string();
        if !cc.is_empty() {
            parsed.confirmation_code = Some(cc);
        }
    }
    
    // Fields 3+: Other parameters
    for (i, part) in parts.iter().skip(3).enumerate() {
        parsed.other.push(LpaField {
            index: i + 3,
            name: None,
            value: part.to_string(),
        });
    }
    
    parsed
}

/// Generate a QR code image from an LPA payload
#[cfg(feature = "qr-encode")]
pub fn generate_qr_image(payload: &str, size: u32) -> Result<Vec<u8>> {
    use qrcode::QrCode;
    use image::Luma;
    
    let code = QrCode::new(payload.as_bytes())
        .map_err(|e| Error::Qr(e.to_string()))?;
    
    let image = code.render::<Luma<u8>>().build();
    let image = image::imageops::resize(
        &image, 
        size, 
        size, 
        image::imageops::FilterType::Nearest
    );
    
    let mut buffer = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buffer);
    
    image::DynamicImage::ImageLuma8(image)
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| Error::Qr(e.to_string()))?;
    
    Ok(buffer)
}

/// Reconstruct LPA payload from parsed fields
pub fn reconstruct_lpa(parsed: &ParsedLpa) -> String {
    let mut parts: Vec<String> = Vec::new();
    
    if let Some(smdp) = &parsed.smdp {
        parts.push(smdp.clone());
    }
    
    if let Some(ac) = &parsed.activation_code {
        parts.push(ac.clone());
    }
    
    if let Some(cc) = &parsed.confirmation_code {
        parts.push(cc.clone());
    }
    
    for field in &parsed.other {
        parts.push(field.value.clone());
    }
    
    if parts.is_empty() {
        String::new()
    } else if let Some(format) = &parsed.format {
        format!("LPA:{}${}", format, parts.join("$"))
    } else {
        parts.join("$")
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn test_parse_various_formats() {
        // Standard format
        let p1 = parse_lpa("LPA:1$sm-dp.example.com$ACTIV123").unwrap();
        assert_eq!(p1.smdp, Some("sm-dp.example.com".to_string()));
        assert_eq!(p1.activation_code, Some("ACTIV123".to_string()));
        
        // Without LPA: prefix
        let p2 = parse_lpa("sm-dp.example.com$ACTIV123").unwrap();
        assert_eq!(p2.smdp, Some("sm-dp.example.com".to_string()));
        
        // Multiple delimiters
        let p3 = parse_lpa("LPA:1$sm-dp.example.com$ACTIV123$CC123$extra").unwrap();
        assert_eq!(p3.confirmation_code, Some("CC123".to_string()));
        assert_eq!(p3.other.len(), 1);
    }

    #[test]
    fn test_reconstruct() {
        let parsed = parse_lpa("LPA:1$sm-dp$ABC$CC").unwrap();
        let reconstructed = reconstruct_lpa(&parsed);
        assert!(reconstructed.contains("sm-dp"));
        assert!(reconstructed.contains("ABC"));
    }
}
