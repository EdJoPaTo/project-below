use std::ffi::OsString;
use std::fs::ReadDir;
use std::path::Path;

pub fn shortened_path(path: &Path, base: &Path) -> Option<String> {
    if !path.starts_with(base) {
        return None;
    }
    let itself = path.file_name()?.to_str()?;
    let mut parts = vec![itself];
    for parent in path.ancestors().skip(1) {
        if parent == base {
            break;
        }
        parts.push(shortest_segment(parent)?);
    }
    parts.reverse();
    let shortened = parts.join(std::path::MAIN_SEPARATOR_STR);
    Some(shortened)
}

fn shortest_segment(path: &Path) -> Option<&str> {
    let itself = path.file_name()?.to_str()?;
    let Some(siblings) = non_file_siblings(path) else {
        return Some(itself);
    };
    let siblings = siblings
        .iter()
        .filter_map(|name| name.to_str())
        .collect::<Vec<_>>();
    Some(shortest_unique(itself, siblings))
}

#[allow(clippy::filetype_is_file)] // links might be relevant
fn non_file_siblings(path: &Path) -> Option<Vec<OsString>> {
    let itself = path.file_name()?;
    let siblings = parent_readdir(path)?
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|ft| !ft.is_file()))
        .map(|entry| entry.file_name())
        .filter(|name| name != itself)
        .collect::<Vec<_>>();
    Some(siblings)
}

fn parent_readdir(path: &Path) -> Option<ReadDir> {
    if let Some(parent) = path.parent() {
        parent.read_dir().ok()
    } else {
        let path = path.canonicalize().ok()?;
        path.parent()?.read_dir().ok()
    }
}

fn shortest_unique<'i>(itself: &'i str, mut others: Vec<&str>) -> &'i str {
    others.retain(|other| !other.starts_with(itself));
    for (index, _) in itself.char_indices().skip(1) {
        #[allow(clippy::string_slice)] // Index from char_indices
        let part = &itself[..index];
        if !part.chars().last().unwrap().is_alphanumeric() {
            continue;
        }
        let is_unique = others.iter().all(|other| !other.starts_with(part));
        if is_unique {
            return part;
        }
    }

    itself
}

#[cfg(test)]
mod tests {
    use super::*;
    #[track_caller]
    fn case(input: &str, expected: &str, others: &[&str]) {
        let result = shortest_unique(input, others.to_vec());
        assert_eq!(result, expected);
        assert!(!result.is_empty(), "Should have at least one character");
    }
    #[test]
    fn short_and_unique() {
        case("a", "a", &["b"]);
    }
    #[test]
    fn shortest_is_unique() {
        case("abc", "a", &["b"]);
    }
    #[test]
    fn no_others() {
        case("abc", "a", &[]);
    }
    #[test]
    fn requires_two_chars() {
        case("abc", "ab", &["asdf"]);
    }
    #[test]
    fn ignores_non_alphanum() {
        case("a-b-c", "a-b", &["a-s-d-f"]);
    }
    #[test]
    fn ignores_longer_of_itself() {
        case("abc", "ab", &["abcd", "asdf"]);
    }
    #[test]
    fn version_numbers_work() {
        case("0.1.2", "0", &[]);
        case("0.1.2", "0.1", &["0.2.0"]);
        case("0.1.2", "0.1.2", &["0.1.3"]);
    }
}
