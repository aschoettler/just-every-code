fn main() {
    // Prefer an explicit CODE_VERSION provided by CI; otherwise use the npm
    // package version so local source builds do not report workspace "0.0.0".
    let version = std::env::var("CODE_VERSION")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(read_release_version_from_package_json)
        .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());

    // Inject the version as a rustc env so it participates in the compiler
    // invocation hash (sccache-friendly) and guarantees a cache miss when
    // the version changes.
    println!("cargo:rustc-env=CODE_VERSION={}", version);

    // Ensure dependent crates rebuild when CODE_VERSION changes even if the
    // source graph stays the same.
    println!("cargo:rerun-if-env-changed=CODE_VERSION");
}

fn read_release_version_from_package_json() -> Option<String> {
    let manifest_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").ok()?);
    let package_json_path = manifest_dir.join("../../codex-cli/package.json");
    let contents = std::fs::read_to_string(package_json_path).ok()?;
    parse_version_from_package_json(&contents)
}

fn parse_version_from_package_json(contents: &str) -> Option<String> {
    for line in contents.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("\"version\"") {
            continue;
        }

        let (_, value_part) = trimmed.split_once(':')?;
        let value_part = value_part.trim();
        let value_part = value_part.strip_prefix('"')?;
        let end = value_part.find('"')?;
        let version = value_part[..end].trim();
        if version.is_empty() {
            return None;
        }
        return Some(version.to_string());
    }

    None
}
