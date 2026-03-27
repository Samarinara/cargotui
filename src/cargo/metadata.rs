use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct MetadataTree {
    pub packages: Vec<PackageInfo>,
}

#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<String>,
}

// Internal serde types for deserializing `cargo metadata --format-version 1`
#[derive(Deserialize)]
struct RawMetadata {
    packages: Vec<RawPackage>,
}

#[derive(Deserialize)]
struct RawPackage {
    name: String,
    version: String,
    dependencies: Vec<RawDependency>,
}

#[derive(Deserialize)]
struct RawDependency {
    name: String,
}

pub fn parse_metadata(json: &str) -> Result<MetadataTree, String> {
    let raw: RawMetadata =
        serde_json::from_str(json).map_err(|e| format!("Failed to parse metadata JSON: {e}"))?;

    let packages = raw
        .packages
        .into_iter()
        .map(|p| PackageInfo {
            name: p.name,
            version: p.version,
            dependencies: p.dependencies.into_iter().map(|d| d.name).collect(),
        })
        .collect();

    Ok(MetadataTree { packages })
}

pub fn format_tree(tree: &MetadataTree) -> Vec<String> {
    let mut lines = Vec::new();
    for pkg in &tree.packages {
        lines.push(format!("  {} v{}", pkg.name, pkg.version));
        for dep in &pkg.dependencies {
            lines.push(format!("    └─ {dep}"));
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_JSON: &str = r#"{
        "packages": [
            {
                "name": "my_crate",
                "version": "0.1.0",
                "dependencies": [
                    { "name": "serde", "req": "^1.0" },
                    { "name": "tokio", "req": "^1" }
                ]
            },
            {
                "name": "helper",
                "version": "0.2.3",
                "dependencies": []
            }
        ]
    }"#;

    #[test]
    fn test_parse_metadata_packages() {
        let tree = parse_metadata(SAMPLE_JSON).unwrap();
        assert_eq!(tree.packages.len(), 2);
        assert_eq!(tree.packages[0].name, "my_crate");
        assert_eq!(tree.packages[0].version, "0.1.0");
        assert_eq!(tree.packages[1].name, "helper");
    }

    #[test]
    fn test_parse_metadata_dependencies() {
        let tree = parse_metadata(SAMPLE_JSON).unwrap();
        assert_eq!(tree.packages[0].dependencies, vec!["serde", "tokio"]);
        assert!(tree.packages[1].dependencies.is_empty());
    }

    #[test]
    fn test_parse_metadata_invalid_json() {
        let result = parse_metadata("not json");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Failed to parse metadata JSON")
        );
    }

    #[test]
    fn test_format_tree_output() {
        let tree = parse_metadata(SAMPLE_JSON).unwrap();
        let lines = format_tree(&tree);
        assert_eq!(lines[0], "  my_crate v0.1.0");
        assert_eq!(lines[1], "    └─ serde");
        assert_eq!(lines[2], "    └─ tokio");
        assert_eq!(lines[3], "  helper v0.2.3");
        assert_eq!(lines.len(), 4);
    }

    #[test]
    fn test_format_tree_empty() {
        let tree = MetadataTree { packages: vec![] };
        assert!(format_tree(&tree).is_empty());
    }

    #[test]
    fn test_parse_metadata_empty_packages() {
        let json = r#"{"packages": []}"#;
        let tree = parse_metadata(json).unwrap();
        assert!(tree.packages.is_empty());
    }
}
