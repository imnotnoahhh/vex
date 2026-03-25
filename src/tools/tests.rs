use super::resolve::{generate_version_suggestions, normalize_version};
use super::*;

#[test]
fn test_arch_detect() {
    let arch = Arch::detect().unwrap();
    match arch {
        Arch::Arm64 | Arch::X86_64 => {}
    }
}

#[test]
fn test_version_struct() {
    let v = Version {
        version: "20.11.0".to_string(),
        lts: Some("Iron".to_string()),
    };
    assert_eq!(v.version, "20.11.0");
    assert_eq!(v.lts, Some("Iron".to_string()));

    let v2 = Version {
        version: "22.0.0".to_string(),
        lts: None,
    };
    assert_eq!(v2.lts, None);
}

#[test]
fn test_get_tool_valid() {
    for name in &["node", "go", "java", "rust", "python"] {
        let tool = get_tool(name);
        assert!(tool.is_ok(), "get_tool({}) should succeed", name);
        assert_eq!(tool.unwrap().name(), *name);
    }
}

#[test]
fn test_get_tool_invalid() {
    let result = get_tool("ruby");
    assert!(result.is_err());

    let result = get_tool("perl");
    assert!(result.is_err());

    let result = get_tool("");
    assert!(result.is_err());
}

struct MockTool {
    versions: Vec<Version>,
}

impl Tool for MockTool {
    fn name(&self) -> &str {
        "mock"
    }

    fn list_remote(&self) -> Result<Vec<Version>> {
        Ok(self.versions.clone())
    }

    fn download_url(&self, _version: &str, _arch: Arch) -> Result<String> {
        Ok(String::new())
    }

    fn checksum_url(&self, _version: &str, _arch: Arch) -> Option<String> {
        None
    }

    fn bin_names(&self) -> Vec<&str> {
        vec!["mock"]
    }

    fn bin_subpath(&self) -> &str {
        "bin"
    }

    fn resolve_alias(&self, alias: &str) -> Result<Option<String>> {
        match alias {
            "latest" => Ok(self.versions.first().map(|v| v.version.clone())),
            "lts" => Ok(self
                .versions
                .iter()
                .find(|v| v.lts.is_some())
                .map(|v| v.version.clone())),
            _ => Ok(None),
        }
    }
}

#[test]
fn test_resolve_fuzzy_version_alias_latest() {
    let tool = MockTool {
        versions: vec![
            Version {
                version: "22.5.0".to_string(),
                lts: None,
            },
            Version {
                version: "20.11.0".to_string(),
                lts: Some("Iron".to_string()),
            },
        ],
    };
    let result = resolve_fuzzy_version(&tool, "latest").unwrap();
    assert_eq!(result, "22.5.0");
}

#[test]
fn test_resolve_fuzzy_version_alias_lts() {
    let tool = MockTool {
        versions: vec![
            Version {
                version: "22.5.0".to_string(),
                lts: None,
            },
            Version {
                version: "20.11.0".to_string(),
                lts: Some("Iron".to_string()),
            },
        ],
    };
    let result = resolve_fuzzy_version(&tool, "lts").unwrap();
    assert_eq!(result, "20.11.0");
}

#[test]
fn test_resolve_fuzzy_version_unknown_alias_falls_through() {
    let tool = MockTool {
        versions: vec![Version {
            version: "22.5.0".to_string(),
            lts: None,
        }],
    };
    let result = resolve_fuzzy_version(&tool, "22.5.0").unwrap();
    assert_eq!(result, "22.5.0");
}

#[test]
fn test_default_resolve_alias_returns_none() {
    struct MinimalTool;

    impl Tool for MinimalTool {
        fn name(&self) -> &str {
            "minimal"
        }

        fn list_remote(&self) -> Result<Vec<Version>> {
            Ok(vec![])
        }

        fn download_url(&self, _: &str, _: Arch) -> Result<String> {
            Ok(String::new())
        }

        fn checksum_url(&self, _: &str, _: Arch) -> Option<String> {
            None
        }

        fn bin_names(&self) -> Vec<&str> {
            vec![]
        }

        fn bin_subpath(&self) -> &str {
            ""
        }
    }

    let tool = MinimalTool;
    assert_eq!(tool.resolve_alias("latest").unwrap(), None);
    assert_eq!(tool.resolve_alias("lts").unwrap(), None);
    assert_eq!(tool.get_checksum("1.0", Arch::Arm64).unwrap(), None);
    assert!(tool
        .post_install(std::path::Path::new("/tmp"), Arch::Arm64)
        .is_ok());
    assert!(tool.bin_paths().is_empty());
}

#[test]
fn test_normalize_version() {
    assert_eq!(normalize_version("v20.11.0"), "20.11.0");
    assert_eq!(normalize_version("20.11.0"), "20.11.0");
    assert_eq!(normalize_version("v1.23"), "1.23");
    assert_eq!(normalize_version("1.23"), "1.23");
}

#[test]
fn test_resolve_fuzzy_version_full_version() {
    let tool = MockTool {
        versions: vec![
            Version {
                version: "20.11.0".to_string(),
                lts: None,
            },
            Version {
                version: "22.5.0".to_string(),
                lts: None,
            },
        ],
    };
    let result = resolve_fuzzy_version(&tool, "20.11.0").unwrap();
    assert_eq!(result, "20.11.0");
}

#[test]
fn test_resolve_fuzzy_version_v_prefix() {
    let tool = MockTool {
        versions: vec![
            Version {
                version: "20.11.0".to_string(),
                lts: None,
            },
            Version {
                version: "22.5.0".to_string(),
                lts: None,
            },
        ],
    };
    let result = resolve_fuzzy_version(&tool, "v20.11.0").unwrap();
    assert_eq!(result, "20.11.0");
}

#[test]
fn test_resolve_fuzzy_version_full_version_not_found() {
    let tool = MockTool {
        versions: vec![
            Version {
                version: "20.11.0".to_string(),
                lts: Some("Iron".to_string()),
            },
            Version {
                version: "22.5.0".to_string(),
                lts: None,
            },
        ],
    };
    let result = resolve_fuzzy_version(&tool, "20.99.0");
    assert!(result.is_err());
    if let Err(crate::error::VexError::VersionNotFound {
        tool,
        version,
        suggestions,
    }) = result
    {
        assert_eq!(tool, "mock");
        assert_eq!(version, "20.99.0");
        assert!(suggestions.contains("Did you mean"));
        assert!(suggestions.contains("20.11.0"));
    } else {
        panic!("Expected VersionNotFound error");
    }
}

#[test]
fn test_resolve_fuzzy_version_partial_match() {
    let tool = MockTool {
        versions: vec![
            Version {
                version: "v22.5.0".to_string(),
                lts: None,
            },
            Version {
                version: "v20.11.0".to_string(),
                lts: Some("Iron".to_string()),
            },
        ],
    };
    let result = resolve_fuzzy_version(&tool, "22").unwrap();
    assert_eq!(result, "22.5.0");
}

#[test]
fn test_resolve_fuzzy_version_no_match() {
    let tool = MockTool {
        versions: vec![Version {
            version: "22.5.0".to_string(),
            lts: None,
        }],
    };
    let result = resolve_fuzzy_version(&tool, "99");
    assert!(result.is_err());
    if let Err(crate::error::VexError::VersionNotFound {
        tool,
        version,
        suggestions,
    }) = result
    {
        assert_eq!(tool, "mock");
        assert_eq!(version, "99");
        assert!(suggestions.contains("Did you mean"));
    }
}

#[test]
fn test_generate_version_suggestions_same_major() {
    let versions = vec![
        Version {
            version: "22.5.0".to_string(),
            lts: None,
        },
        Version {
            version: "20.11.0".to_string(),
            lts: Some("Iron".to_string()),
        },
        Version {
            version: "20.10.0".to_string(),
            lts: None,
        },
    ];
    let suggestions = generate_version_suggestions("20.99.0", &versions);
    assert!(suggestions.contains("20.11.0"));
    assert!(suggestions.contains("latest in 20.x"));
}

#[test]
fn test_generate_version_suggestions_same_minor() {
    let versions = vec![
        Version {
            version: "20.11.5".to_string(),
            lts: None,
        },
        Version {
            version: "20.11.0".to_string(),
            lts: Some("Iron".to_string()),
        },
        Version {
            version: "20.10.0".to_string(),
            lts: None,
        },
    ];
    let suggestions = generate_version_suggestions("20.11.99", &versions);
    assert!(suggestions.contains("20.11.5"));
    assert!(suggestions.contains("latest in 20.11.x"));
}

#[test]
fn test_generate_version_suggestions_nearby() {
    let versions = vec![
        Version {
            version: "22.5.0".to_string(),
            lts: None,
        },
        Version {
            version: "21.0.0".to_string(),
            lts: None,
        },
        Version {
            version: "20.11.0".to_string(),
            lts: Some("Iron".to_string()),
        },
    ];
    let suggestions = generate_version_suggestions("19.0.0", &versions);
    assert!(suggestions.contains("20.11.0") || suggestions.contains("21.0.0"));
}

#[test]
fn test_generate_version_suggestions_latest() {
    let versions = vec![
        Version {
            version: "22.5.0".to_string(),
            lts: None,
        },
        Version {
            version: "20.11.0".to_string(),
            lts: Some("Iron".to_string()),
        },
    ];
    let suggestions = generate_version_suggestions("99.0.0", &versions);
    assert!(suggestions.contains("22.5.0"));
    assert!(suggestions.contains("(latest)"));
}

#[test]
fn test_generate_version_suggestions_empty() {
    let versions = vec![];
    let suggestions = generate_version_suggestions("20.0.0", &versions);
    assert!(suggestions.is_empty());
}
