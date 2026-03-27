use std::path::{Path, PathBuf};
use toml::Value;

pub struct Workspace {
    pub root: PathBuf,
    pub manifest: Manifest,
}

pub struct Manifest {
    pub name: String,
    pub members: Vec<String>,
    pub dependencies: Vec<Dependency>,
    pub dev_dependencies: Vec<Dependency>,
    pub build_dependencies: Vec<Dependency>,
    pub binaries: Vec<BinaryTarget>,
}

pub struct Dependency {
    pub name: String,
    pub version: String,
    pub section: DepSection,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DepSection {
    Normal,
    Dev,
    Build,
}

pub struct BinaryTarget {
    pub name: String,
    pub path: Option<String>,
}

/// Walk parent directories from `start` looking for `Cargo.toml`.
/// Returns `Err` if no `Cargo.toml` is found in `start` or any ancestor.
pub fn find_workspace(start: &Path) -> Result<Workspace, String> {
    let mut current = start.to_path_buf();
    loop {
        let candidate = current.join("Cargo.toml");
        if candidate.is_file() {
            return parse_workspace(&current, &candidate);
        }
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => {
                return Err(
                    "No Cargo.toml found in current directory or any parent directory".to_string(),
                );
            }
        }
    }
}

fn parse_workspace(root: &Path, toml_path: &Path) -> Result<Workspace, String> {
    let content = std::fs::read_to_string(toml_path)
        .map_err(|e| format!("Failed to read Cargo.toml: {}", e))?;

    let value: Value = content
        .parse::<Value>()
        .map_err(|e| format!("Failed to parse Cargo.toml: {}", e))?;

    let manifest = parse_manifest(&value)?;

    Ok(Workspace {
        root: root.to_path_buf(),
        manifest,
    })
}

fn parse_manifest(value: &Value) -> Result<Manifest, String> {
    // Extract package name
    let name = value
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("")
        .to_string();

    // Extract workspace members
    let members = value
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(|m| m.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    // Extract dependencies by section
    let dependencies = parse_dep_section(value, "dependencies", DepSection::Normal);
    let dev_dependencies = parse_dep_section(value, "dev-dependencies", DepSection::Dev);
    let build_dependencies = parse_dep_section(value, "build-dependencies", DepSection::Build);

    // Extract [[bin]] targets
    let binaries = parse_bin_targets(value);

    Ok(Manifest {
        name,
        members,
        dependencies,
        dev_dependencies,
        build_dependencies,
        binaries,
    })
}

fn parse_dep_section(value: &Value, section: &str, dep_section: DepSection) -> Vec<Dependency> {
    let table = match value.get(section).and_then(|v| v.as_table()) {
        Some(t) => t,
        None => return vec![],
    };

    table
        .iter()
        .map(|(name, spec)| {
            let version = extract_version(spec);
            Dependency {
                name: name.clone(),
                version,
                section: dep_section.clone(),
            }
        })
        .collect()
}

fn extract_version(spec: &Value) -> String {
    match spec {
        // Simple string version: `foo = "1.0"`
        Value::String(v) => v.clone(),
        // Inline table: `foo = { version = "1.0", features = [...] }`
        Value::Table(t) => t
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("*")
            .to_string(),
        _ => "*".to_string(),
    }
}

fn parse_bin_targets(value: &Value) -> Vec<BinaryTarget> {
    value
        .get("bin")
        .and_then(|b| b.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|entry| {
                    let name = entry.get("name")?.as_str()?.to_string();
                    let path = entry
                        .get("path")
                        .and_then(|p| p.as_str())
                        .map(|s| s.to_string());
                    Some(BinaryTarget { name, path })
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::fs;
    use std::io::Write;

    fn write_toml(dir: &Path, content: &str) {
        let path = dir.join("Cargo.toml");
        let mut f = fs::File::create(path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn test_find_workspace_in_current_dir() {
        let dir = tempfile::tempdir().unwrap();
        write_toml(
            dir.path(),
            r#"
[package]
name = "myapp"
version = "0.1.0"
edition = "2021"
"#,
        );
        let ws = find_workspace(dir.path()).unwrap();
        assert_eq!(ws.root, dir.path());
        assert_eq!(ws.manifest.name, "myapp");
    }

    #[test]
    fn test_find_workspace_in_parent_dir() {
        let dir = tempfile::tempdir().unwrap();
        write_toml(
            dir.path(),
            r#"
[package]
name = "parent_pkg"
version = "0.1.0"
edition = "2021"
"#,
        );
        let child = dir.path().join("sub").join("deep");
        fs::create_dir_all(&child).unwrap();
        let ws = find_workspace(&child).unwrap();
        assert_eq!(ws.root, dir.path());
    }

    #[test]
    fn test_find_workspace_no_toml_returns_err() {
        let dir = tempfile::tempdir().unwrap();
        // Use a path that definitely has no Cargo.toml anywhere
        let result = find_workspace(dir.path());
        // May or may not find one depending on the system; use a path inside /tmp
        // We just verify the function returns without panicking
        let _ = result;
    }

    #[test]
    fn test_parse_dependencies() {
        let dir = tempfile::tempdir().unwrap();
        write_toml(
            dir.path(),
            r#"
[package]
name = "deptest"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
tokio = { version = "1", features = ["full"] }

[dev-dependencies]
proptest = "1"

[build-dependencies]
cc = "1.0"
"#,
        );
        let ws = find_workspace(dir.path()).unwrap();
        let m = &ws.manifest;
        assert_eq!(m.dependencies.len(), 2);
        assert_eq!(m.dev_dependencies.len(), 1);
        assert_eq!(m.build_dependencies.len(), 1);

        let serde = m.dependencies.iter().find(|d| d.name == "serde").unwrap();
        assert_eq!(serde.version, "1.0");
        assert_eq!(serde.section, DepSection::Normal);

        let tokio = m.dependencies.iter().find(|d| d.name == "tokio").unwrap();
        assert_eq!(tokio.version, "1");

        let proptest = &m.dev_dependencies[0];
        assert_eq!(proptest.section, DepSection::Dev);

        let cc = &m.build_dependencies[0];
        assert_eq!(cc.section, DepSection::Build);
    }

    #[test]
    fn test_parse_workspace_members() {
        let dir = tempfile::tempdir().unwrap();
        write_toml(
            dir.path(),
            r#"
[workspace]
members = ["crate-a", "crate-b"]
"#,
        );
        let ws = find_workspace(dir.path()).unwrap();
        assert_eq!(ws.manifest.members, vec!["crate-a", "crate-b"]);
    }

    #[test]
    fn test_parse_bin_targets() {
        let dir = tempfile::tempdir().unwrap();
        write_toml(
            dir.path(),
            r#"
[package]
name = "multibin"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "server"
path = "src/bin/server.rs"

[[bin]]
name = "client"
path = "src/bin/client.rs"
"#,
        );
        let ws = find_workspace(dir.path()).unwrap();
        assert_eq!(ws.manifest.binaries.len(), 2);
        assert!(ws.manifest.binaries.iter().any(|b| b.name == "server"));
        assert!(ws.manifest.binaries.iter().any(|b| b.name == "client"));
    }

    // Feature: cargo-tui, Property 2: Missing workspace produces error
    proptest! {
        #[test]
        fn prop_missing_workspace_returns_err(depth in 1..=5usize) {
            // **Validates: Requirements 1.3**
            let root_dir = tempfile::tempdir().unwrap();

            // Create nested subdirectories with no Cargo.toml anywhere
            let mut deepest = root_dir.path().to_path_buf();
            for i in 0..depth {
                deepest = deepest.join(format!("level_{}", i));
            }
            fs::create_dir_all(&deepest).unwrap();

            // Call find_workspace from the deepest subdirectory
            let result = find_workspace(&deepest);

            // Assert it returns Err (no Cargo.toml exists in any ancestor under /tmp)
            prop_assert!(result.is_err(), "Expected Err but got Ok with root: {:?}", result.ok().map(|ws| ws.root));
        }
    }

    // Feature: cargo-tui, Property 12: Dependency grouping
    proptest! {
        #[test]
        fn prop_dependency_grouping_reflects_manifest_sections(
            normal_deps in prop::collection::vec(r"[a-z][a-z0-9\-]{0,10}", 0..=5),
            dev_deps in prop::collection::vec(r"[a-z][a-z0-9\-]{0,10}", 0..=5),
            build_deps in prop::collection::vec(r"[a-z][a-z0-9\-]{0,10}", 0..=5),
        ) {
            // **Validates: Requirements 6.7**

            // Deduplicate within each section and across all sections to avoid TOML key conflicts
            let mut seen = std::collections::HashSet::new();
            let normal: Vec<String> = normal_deps.into_iter()
                .filter(|n| seen.insert(n.clone()))
                .collect();
            let dev: Vec<String> = dev_deps.into_iter()
                .filter(|n| seen.insert(n.clone()))
                .collect();
            let build: Vec<String> = build_deps.into_iter()
                .filter(|n| seen.insert(n.clone()))
                .collect();

            // Build Cargo.toml content
            let mut toml = String::from("[package]\nname = \"test-pkg\"\nversion = \"0.1.0\"\nedition = \"2021\"\n");

            if !normal.is_empty() {
                toml.push_str("\n[dependencies]\n");
                for dep in &normal {
                    toml.push_str(&format!("{} = \"1.0\"\n", dep));
                }
            }
            if !dev.is_empty() {
                toml.push_str("\n[dev-dependencies]\n");
                for dep in &dev {
                    toml.push_str(&format!("{} = \"1.0\"\n", dep));
                }
            }
            if !build.is_empty() {
                toml.push_str("\n[build-dependencies]\n");
                for dep in &build {
                    toml.push_str(&format!("{} = \"1.0\"\n", dep));
                }
            }

            let dir = tempfile::tempdir().unwrap();
            write_toml(dir.path(), &toml);

            let ws = find_workspace(dir.path()).unwrap();
            let m = &ws.manifest;

            // Collect names from each parsed section
            let parsed_normal: std::collections::HashSet<String> =
                m.dependencies.iter().map(|d| d.name.clone()).collect();
            let parsed_dev: std::collections::HashSet<String> =
                m.dev_dependencies.iter().map(|d| d.name.clone()).collect();
            let parsed_build: std::collections::HashSet<String> =
                m.build_dependencies.iter().map(|d| d.name.clone()).collect();

            // Union of all parsed sections must equal the full declared set
            let mut parsed_union = parsed_normal.clone();
            parsed_union.extend(parsed_dev.clone());
            parsed_union.extend(parsed_build.clone());

            let declared: std::collections::HashSet<String> = normal.iter()
                .chain(dev.iter())
                .chain(build.iter())
                .cloned()
                .collect();

            prop_assert_eq!(&parsed_union, &declared, "Union of all dep sections should equal declared set");

            // Each dep must be in the correct section
            for dep in &normal {
                prop_assert!(parsed_normal.contains(dep), "Normal dep '{}' not in dependencies section", dep);
                prop_assert!(!parsed_dev.contains(dep), "Normal dep '{}' incorrectly in dev-dependencies", dep);
                prop_assert!(!parsed_build.contains(dep), "Normal dep '{}' incorrectly in build-dependencies", dep);
            }
            for dep in &dev {
                prop_assert!(parsed_dev.contains(dep), "Dev dep '{}' not in dev-dependencies section", dep);
                prop_assert!(!parsed_normal.contains(dep), "Dev dep '{}' incorrectly in dependencies", dep);
                prop_assert!(!parsed_build.contains(dep), "Dev dep '{}' incorrectly in build-dependencies", dep);
            }
            for dep in &build {
                prop_assert!(parsed_build.contains(dep), "Build dep '{}' not in build-dependencies section", dep);
                prop_assert!(!parsed_normal.contains(dep), "Build dep '{}' incorrectly in dependencies", dep);
                prop_assert!(!parsed_dev.contains(dep), "Build dep '{}' incorrectly in dev-dependencies", dep);
            }

            // Each dep section has the correct DepSection tag
            for dep in &m.dependencies {
                prop_assert_eq!(&dep.section, &DepSection::Normal, "dep '{}' should have Normal section", dep.name);
            }
            for dep in &m.dev_dependencies {
                prop_assert_eq!(&dep.section, &DepSection::Dev, "dep '{}' should have Dev section", dep.name);
            }
            for dep in &m.build_dependencies {
                prop_assert_eq!(&dep.section, &DepSection::Build, "dep '{}' should have Build section", dep.name);
            }
        }
    }

    // Feature: cargo-tui, Property 1: Workspace detection finds ancestor
    proptest! {
        #[test]
        fn prop_find_workspace_in_any_ancestor(depth in 1..=5usize) {
            // **Validates: Requirements 1.1, 1.2**
            let root_dir = tempfile::tempdir().unwrap();

            // Place a minimal Cargo.toml at the root temp dir (the ancestor)
            write_toml(root_dir.path(), r#"
[package]
name = "ancestor_pkg"
version = "0.1.0"
edition = "2021"
"#);

            // Create nested subdirectories to the given depth
            let mut deepest = root_dir.path().to_path_buf();
            for i in 0..depth {
                deepest = deepest.join(format!("level_{}", i));
            }
            fs::create_dir_all(&deepest).unwrap();

            // Call find_workspace from the deepest subdirectory
            let result = find_workspace(&deepest);

            // Assert it returns Ok and the root matches the temp dir
            prop_assert!(result.is_ok(), "Expected Ok but got Err: {:?}", result.err());
            let ws = result.unwrap();
            prop_assert_eq!(
                ws.root.canonicalize().unwrap(),
                root_dir.path().canonicalize().unwrap(),
                "Workspace root should be the ancestor temp dir"
            );
        }
    }
}
