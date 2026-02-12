const SENSITIVE_KEYS: &[&str] = &[
    "token",
    "access_token",
    "appsecret",
    "authorization",
    "cookie",
    "passwd",
    "password",
    "secret",
];

pub(crate) fn redact_text(input: &str) -> String {
    let mut output = input.to_owned();
    for key in SENSITIVE_KEYS {
        let lower = output.to_lowercase();
        if let Some(index) = lower.find(key) {
            let start = index + key.len();
            let end = (start + 32).min(output.len());
            if start < end {
                output.replace_range(start..end, "=<redacted>");
            }
        }
    }
    output
}

pub(crate) fn truncate_snippet(input: &str, max_bytes: usize) -> String {
    if input.len() <= max_bytes {
        return input.to_string();
    }
    let mut end = max_bytes;
    while !input.is_char_boundary(end) {
        end -= 1;
    }
    let mut value = input[..end].to_string();
    value.push_str("...(truncated)");
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_snippet_keeps_boundary() {
        let text = "hello world";
        assert_eq!(truncate_snippet(text, 5), "hello...(truncated)");
    }
}
