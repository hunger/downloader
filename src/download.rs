// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (C) 2020 Tobias Hunger <tobias.hunger@gmail.com>

//! The implementation of the `Download` struct

// ----------------------------------------------------------------------
// - Download:
// ----------------------------------------------------------------------

/// A Progress reporter
pub type Progress = std::sync::Arc<dyn crate::progress::Reporter>;

/// A simple progress callback passed to `VerifyCallback`
pub type SimpleProgress = dyn Fn(u64) + Sync;

/// A callback to used to verify the download.
pub type Verify = std::sync::Arc<dyn Fn(std::path::PathBuf, &SimpleProgress) -> bool + Send + Sync>;

/// A `Download` to be run.
pub struct Download {
    /// A list of URLs that this file can be retrieved from
    pub urls: Vec<String>,
    /// A progress `Reporter` to report the download process with.
    pub progress: Option<Progress>,
    /// The file name to be used for the downloaded file.
    pub file_name: std::path::PathBuf,
    /// A callback used to verify the download with.
    pub verify_callback: Verify,
}

fn file_name_from_url(url: &str) -> std::path::PathBuf {
    if url.is_empty() {
        return std::path::PathBuf::new();
    }
    let url = match reqwest::Url::parse(url) {
        Ok(u) => u,
        Err(_) => return std::path::PathBuf::new(),
    };

    let url_file = url.path_segments();
    match url_file {
        Some(f) => std::path::PathBuf::from(f.last().unwrap_or("")),
        None => std::path::PathBuf::new(),
    }
}

impl Download {
    /// Create a new `Download` with a single download `url`
    #[must_use]
    pub fn new(url: &str) -> Self {
        Self {
            urls: vec![url.to_owned()],
            progress: None,
            file_name: file_name_from_url(url),
            verify_callback: crate::verify::noop(),
        }
    }

    /// Create a new `Download` based on a list of mirrors
    #[must_use]
    pub fn new_mirrored(urls: &[&str]) -> Self {
        let urls: Vec<String> = urls.iter().map(|s| String::from(*s)).collect();
        let url = urls.get(0).unwrap_or(&String::new()).clone();

        Self {
            urls,
            progress: None,
            file_name: file_name_from_url(&url),
            verify_callback: crate::verify::noop(),
        }
    }

    /// Register a callback to provide progress information
    ///
    /// Default is the file name on the server side (if available)
    #[must_use]
    pub fn file_name(mut self, path: &std::path::Path) -> Self {
        self.file_name = path.to_owned();
        self
    }

    /// Register handling of progress information
    ///
    /// Defaults to not printing any progress infromation.
    #[must_use]
    pub fn progress(mut self, progress: Progress) -> Self {
        self.progress = Some(progress);
        self
    }

    /// Register a callback to verify a download
    ///
    /// Default is to assume the file was downloaded correctly.
    #[must_use]
    pub fn verify(mut self, func: Verify) -> Self {
        self.verify_callback = func;
        self
    }
}
