const ANNOUNCEMENT_TIP: &str = include_str!("../../../announcement_tip.toml");

pub(crate) fn backend_compatible_version(current_version: &str) -> String {
    let minimum = extract_first_semver(ANNOUNCEMENT_TIP);
    backend_compatible_version_with_minimum(current_version, minimum.as_deref())
}

fn backend_compatible_version_with_minimum(
    current_version: &str,
    minimum_version: Option<&str>,
) -> String {
    let Some(minimum_version) = minimum_version else {
        return current_version.to_string();
    };

    match (
        parse_semver_triplet(current_version),
        parse_semver_triplet(minimum_version),
    ) {
        (Some(left_triplet), Some(right_triplet)) => {
            if left_triplet >= right_triplet {
                current_version.to_string()
            } else {
                minimum_version.to_string()
            }
        }
        _ => current_version.to_string(),
    }
}

fn extract_first_semver(input: &str) -> Option<String> {
    for token in input.split_whitespace() {
        let candidate = token.trim_matches(|ch: char| {
            !(ch.is_ascii_alphanumeric() || ch == '.' || ch == '-' || ch == '+' || ch == 'v')
        });
        if candidate.is_empty() {
            continue;
        }
        if parse_semver_triplet(candidate).is_some() {
            return Some(candidate.to_string());
        }
    }

    None
}

fn parse_semver_triplet(version: &str) -> Option<(u64, u64, u64)> {
    let trimmed = version.trim().trim_start_matches('v');
    let core = trimmed
        .split_once('+')
        .map_or(trimmed, |(value, _)| value);
    let core = core
        .split_once('-')
        .map_or(core, |(value, _)| value);

    let mut parts = core.split('.');
    let major = parse_numeric_component(parts.next()?)?;
    let minor = parse_numeric_component(parts.next()?)?;
    let patch = parse_numeric_component(parts.next()?)?;

    if parts.next().is_some() {
        return None;
    }

    Some((major, minor, patch))
}

fn parse_numeric_component(component: &str) -> Option<u64> {
    if component.is_empty() || !component.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }

    component.parse::<u64>().ok()
}

#[cfg(test)]
mod tests {
    use super::{backend_compatible_version_with_minimum, extract_first_semver};

    #[test]
    fn floors_older_versions_to_minimum_backend_version() {
        assert_eq!(
            backend_compatible_version_with_minimum("0.6.59", Some("0.98.0")),
            "0.98.0"
        );
        assert_eq!(
            backend_compatible_version_with_minimum("0.98.0-beta.1", Some("0.98.0")),
            "0.98.0-beta.1"
        );
    }

    #[test]
    fn preserves_newer_versions() {
        assert_eq!(
            backend_compatible_version_with_minimum("0.99.0", Some("0.98.0")),
            "0.99.0"
        );
        assert_eq!(
            backend_compatible_version_with_minimum("1.2.3", Some("0.98.0")),
            "1.2.3"
        );
    }

    #[test]
    fn keeps_input_when_version_is_not_semver() {
        assert_eq!(
            backend_compatible_version_with_minimum("dev-build", Some("0.98.0")),
            "dev-build"
        );
    }

    #[test]
    fn extracts_semver_from_announcement_text() {
        let announcement = "Upgrade to `0.98.0` for gpt-5.3-codex.";
        assert_eq!(extract_first_semver(announcement).as_deref(), Some("0.98.0"));
    }
}
