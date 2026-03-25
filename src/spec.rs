use crate::error::{Result, VexError};

pub fn parse_spec(spec: &str) -> Result<(String, String)> {
    if let Some((tool_name, version)) = spec.split_once('@') {
        if spec.matches('@').count() > 1 {
            Err(VexError::Parse(format!(
                "Invalid spec format: {}. Expected format: tool@version or tool",
                spec
            )))
        } else {
            Ok((tool_name.to_string(), version.to_string()))
        }
    } else {
        Ok((spec.to_string(), "".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_spec_with_version() {
        let (tool, version) = parse_spec("node@20.11.0").unwrap();
        assert_eq!(tool, "node");
        assert_eq!(version, "20.11.0");
    }

    #[test]
    fn test_parse_spec_tool_only() {
        let (tool, version) = parse_spec("node").unwrap();
        assert_eq!(tool, "node");
        assert_eq!(version, "");
    }

    #[test]
    fn test_parse_spec_java() {
        let (tool, version) = parse_spec("java@21").unwrap();
        assert_eq!(tool, "java");
        assert_eq!(version, "21");
    }

    #[test]
    fn test_parse_spec_rust() {
        let (tool, version) = parse_spec("rust@1.93.1").unwrap();
        assert_eq!(tool, "rust");
        assert_eq!(version, "1.93.1");
    }

    #[test]
    fn test_parse_spec_go() {
        let (tool, version) = parse_spec("go@1.23.5").unwrap();
        assert_eq!(tool, "go");
        assert_eq!(version, "1.23.5");
    }

    #[test]
    fn test_parse_spec_invalid_multiple_at() {
        let result = parse_spec("node@20@11");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_spec_empty_version() {
        let (tool, version) = parse_spec("node@").unwrap();
        assert_eq!(tool, "node");
        assert_eq!(version, "");
    }

    #[test]
    fn test_parse_spec_version_with_v_prefix() {
        let (tool, version) = parse_spec("node@v20.11.0").unwrap();
        assert_eq!(tool, "node");
        assert_eq!(version, "v20.11.0");
    }

    #[test]
    fn test_parse_spec_error_message() {
        let err = parse_spec("node@20@11").unwrap_err();
        assert!(err.to_string().contains("Invalid spec format"));
    }
}
