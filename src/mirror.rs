use url::{Url, ParseError};
use std::collections::HashSet;

pub struct MirrorContext {
    base_urls: Vec<Url>,
}

impl MirrorContext {
    pub fn from_urls<T: AsRef<str>>(base_urls: &[T]) -> Result<Self, ParseError> {
        let mut normalized_urls = Vec::new();

        for base_url in base_urls {
            normalized_urls.push(Url::parse(base_url.as_ref())?);
        }

        let ctx = Self {
            base_urls: normalized_urls,
        };

        Ok(ctx)
    }

    pub fn urls_for_file(&self, relative_path: &str) -> Result<Vec<String>, ParseError> {
        let mut built_urls = Vec::new();

        for base_url in &self.base_urls {
            let new_url = base_url.join(relative_path)?.to_string();
            built_urls.push(new_url);
        }

        Ok(built_urls)
    }
}



#[cfg(test)]
mod tests {
    use super::MirrorContext;
    use std::error::Error;

    static MIRRORS: &[&str] = &[
        "http://ftp.au.debian.org/debian/",
        "http://ftp.ca.debian.org/debian/",
        "http://ftp.cz.debian.org/debian/",
        "http://ftp.us.debian.org/debian/",
    ];

    #[test]
    fn test_mirror_urls() -> Result<(), Box<dyn Error>> {
        let debian_mirrors = MirrorContext::from_urls(MIRRORS)?;

        assert_eq!(
            &debian_mirrors.urls_for_file("README.txt")?,
            &[
                "http://ftp.au.debian.org/debian/README.txt",
                "http://ftp.ca.debian.org/debian/README.txt",
                "http://ftp.cz.debian.org/debian/README.txt",
                "http://ftp.us.debian.org/debian/README.txt",
            ]
        );

        Ok(())
    }
}
