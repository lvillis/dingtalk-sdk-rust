use url::Url;

use crate::error::{Error, Result};

pub(crate) fn normalize_base_url(value: impl AsRef<str>) -> Result<Url> {
    let raw = value.as_ref().trim();
    let mut url = Url::parse(raw).map_err(|source| Error::InvalidConfig {
        message: format!("Invalid base_url `{raw}`"),
        source: Some(Box::new(source)),
    })?;

    if url.cannot_be_a_base() {
        return Err(Error::InvalidConfig {
            message: "base_url must be hierarchical".to_string(),
            source: None,
        });
    }

    if url.query().is_some() || url.fragment().is_some() {
        return Err(Error::InvalidConfig {
            message: "base_url must not contain query or fragment".to_string(),
            source: None,
        });
    }

    if url.path() != "/" {
        let path = url.path().trim_end_matches('/').to_string();
        if path.is_empty() {
            url.set_path("/");
        } else {
            url.set_path(&path);
        }
    }

    Ok(url)
}

pub(crate) fn endpoint_url(base_url: &Url, segments: &[&str]) -> Result<Url> {
    let mut url = base_url.clone();
    {
        let mut path_segments = url.path_segments_mut().map_err(|_| Error::InvalidConfig {
            message: "base_url must be hierarchical".to_string(),
            source: None,
        })?;
        for segment in segments {
            path_segments.push(segment);
        }
    }
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_url_encodes_segments() {
        let base = normalize_base_url("https://example.com/api").expect("base");
        let url = endpoint_url(&base, &["users", "a/b"]).expect("url");
        assert_eq!(url.as_str(), "https://example.com/api/users/a%2Fb");
    }

    #[test]
    fn normalize_base_url_rejects_query() {
        let error = normalize_base_url("https://example.com/api?x=1").expect_err("must fail");
        match error {
            Error::InvalidConfig { .. } => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
