use regex::Regex;

pub fn custom_strip_prefix<'a>(s: &'a str, prefix: &str) -> &'a str {
    if s.starts_with(prefix) {
        &s[prefix.len()..]
    } else {
        s
    }
}

pub fn custom_strip_suffix<'a>(s: &'a str, suffix: &str) -> &'a str {
    if s.ends_with(suffix) {
        let suffix_len = suffix.len();
        &s[..s.len() - suffix_len]
    } else {
        s
    }
}

pub fn custom_strip<'a>(s: &'a str, replace: &str) -> &'a str {
    custom_strip_prefix(custom_strip_suffix(s, replace), replace)
}

pub fn wildcard_match(pattern: &str, input: &str) -> bool {
    let pattern = pattern.replace("%", ".*").replace("_", ".");
    let regex = Regex::new(&format!("^{}$", pattern)).unwrap();
    regex.is_match(input)
}
