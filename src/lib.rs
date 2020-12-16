// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (C) 2020 Tobias Hunger <tobias.hunger@gmail.com>

//! A simple way to download files via HTTP/HTTPS

// Setup warnings/errors:
#![forbid(unsafe_code)]
#![deny(
    bare_trait_objects,
    unused_doc_comments,
    unused_import_braces,
    missing_docs
)]
// Clippy:
#![warn(clippy::all, clippy::nursery, clippy::pedantic)]
#![allow(clippy::non_ascii_literal)]

pub mod download;

// ----------------------------------------------------------------------
// - Error:
// ----------------------------------------------------------------------

/// An `Error` Enum
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// The Setup is incomplete or bogus.
    #[error("Setup error: {0}")]
    Setup(String),
    /// The backend crate reported some issue.
    #[error("Backend error: {0}")]
    Backend(#[from] reqwest::Error),
}

/// `Result` type for the `gng_shared` library
pub type Result<T> = std::result::Result<T, Error>;

// ----------------------------------------------------------------------
// - Download:
// ----------------------------------------------------------------------

/// A callback to report download progress. The inputs are the current
/// retry count, the maximum retry count, the downloaded size in bytes
/// and the total file size in bytes.
type ProgressCallback = Box<dyn Fn(u16, u16, u64, Option<u64>) + Send + Sync>;

/// A `Download` to be run.
pub struct Download {
    urls: Vec<String>,
    callback: ProgressCallback,
    file_name: std::path::PathBuf,
}

fn noop_callback() -> ProgressCallback {
    Box::new(|_: u16, _: u16, _: u64, _: Option<u64>| {})
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
            callback: noop_callback(),
            file_name: file_name_from_url(url),
        }
    }

    /// Create a new `Download` based on a list of mirrors
    #[must_use]
    pub fn new_mirrored(urls: Vec<&str>) -> Self {
        let empty = String::new();
        let urls: Vec<String> = urls
            .into_iter()
            .map(std::borrow::ToOwned::to_owned)
            .collect();
        let url = urls.get(0).unwrap_or(&empty).clone();

        Self {
            urls,
            callback: noop_callback(),
            file_name: file_name_from_url(&url),
        }
    }

    /// Register a callback to provide progress information
    #[must_use]
    pub fn file_name(mut self, path: &std::path::Path) -> Self {
        self.file_name = path.to_owned();
        self
    }

    /// Register a callback to provide progress information
    #[must_use]
    pub fn callback(mut self, func: ProgressCallback) -> Self {
        self.callback = func;
        self
    }
}

// ----------------------------------------------------------------------
// - DownloadResult:
// ----------------------------------------------------------------------

/// The result of a `Download`
pub struct DownloadResult {
    /// The actual URL that this file has been downloaded from
    pub status: Vec<(String, u16)>,
    /// The path this URL has been downloaded to.
    pub file_name: std::path::PathBuf,
}

impl DownloadResult {
    /// Returns whether this was a successful download or not.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.status.last().unwrap_or(&(String::from(""), 0)).1 == 200
    }
}

// ----------------------------------------------------------------------
// - Downloader:
// ----------------------------------------------------------------------

/// The main entry point
pub struct Downloader {
    client: reqwest::Client,
    downloads: Vec<Download>,
    parallel_requests: u16,
    retries: u16,
    download_folder: std::path::PathBuf,
}

impl Downloader {
    /// Create a builder for `Downloader`
    #[must_use]
    pub fn builder() -> DownloaderBuilder {
        let download_folder =
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(""));
        let download_folder = if download_folder.to_string_lossy().is_empty() {
            std::path::PathBuf::from(
                std::env::var_os("HOME").unwrap_or_else(|| std::ffi::OsString::from("/")),
            )
        } else {
            download_folder
        };

        DownloaderBuilder {
            user_agent: format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
            connect_timeout: std::time::Duration::from_secs(30),
            timeout: std::time::Duration::from_secs(300),
            parallel_requests: 32,
            retries: 3,
            download_folder,
        }
    }

    /// Queue a `Download`
    pub fn queue(&mut self, download: Download) {
        self.downloads.push(download);
    }

    /// Start the download
    ///
    /// # Errors
    /// `Error::Setup` if the download is detected to be broken in some way.
    pub fn download(&mut self) -> Result<Vec<DownloadResult>> {
        let mut to_process = std::mem::take(&mut self.downloads);

        let mut known_urls = std::collections::HashSet::new();
        let mut known_download_paths = std::collections::HashSet::new();

        for d in &mut to_process {
            if d.urls.is_empty() {
                return Err(Error::Setup(String::from("No URL found to download.")));
            }

            for u in &d.urls {
                if !known_urls.insert(u) {
                    return Err(Error::Setup(format!(
                        "Download URL \"{}\" is used more than once.",
                        u
                    )));
                }
            }

            d.file_name = self.download_folder.join(&d.file_name);
            if d.file_name.to_string_lossy().is_empty() {
                return Err(Error::Setup(String::from(
                    "Failed to get full download path.",
                )));
            }

            if !known_download_paths.insert(&d.file_name) {
                return Err(Error::Setup(format!(
                    "Download file name \"{}\" is used more than once.",
                    d.file_name.to_string_lossy(),
                )));
            }
        }

        Ok(download::run(
            &mut self.client,
            to_process,
            self.retries,
            self.parallel_requests,
        ))
    }
}

// ----------------------------------------------------------------------
// - DownloadBuilder:
// ----------------------------------------------------------------------

/// A builder for `Downloader`
pub struct DownloaderBuilder {
    user_agent: String,
    connect_timeout: std::time::Duration,
    timeout: std::time::Duration,
    parallel_requests: u16,
    retries: u16,
    download_folder: std::path::PathBuf,
}

impl DownloaderBuilder {
    /// Set the user agent to be used.
    ///
    /// A default value will be used if none is set.
    pub fn user_agent(&mut self, user_agent: &str) -> &mut Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Set the connection timeout.
    ///
    /// The default is 30s.
    pub fn connect_timeout(&mut self, timeout: std::time::Duration) -> &mut Self {
        self.connect_timeout = timeout;
        self
    }

    /// Set the timeout.
    ///
    /// The default is 5min.
    pub fn timeout(&mut self, timeout: std::time::Duration) -> &mut Self {
        self.timeout = timeout;
        self
    }

    /// Set the number of parallel requests.
    ///
    /// The default is 32.
    pub fn parallel_requests(&mut self, count: u16) -> &mut Self {
        self.parallel_requests = count;
        self
    }

    /// Set the number of retries.
    ///
    /// The default is 3.
    pub fn retries(&mut self, count: u16) -> &mut Self {
        self.retries = count;
        self
    }

    /// Set the folder to download into.
    ///
    /// The default is unset and a value is required.
    pub fn download_folder(&mut self, folder: &std::path::Path) -> &mut Self {
        self.download_folder = folder.to_path_buf();
        self
    }

    /// Build a downloader.
    ///
    /// # Errors
    /// * `Error::Setup`, when `reqwest::Client` setup fails
    pub fn build(&mut self) -> Result<Downloader> {
        let builder = reqwest::Client::builder()
            .user_agent(self.user_agent.clone())
            .connect_timeout(self.connect_timeout)
            .timeout(self.timeout);

        let download_folder = &self.download_folder;
        if download_folder.to_string_lossy().is_empty() {
            return Err(Error::Setup(
                "Required \"download_folder\" was not set.".into(),
            ));
        }
        if !download_folder.is_dir() {
            return Err(Error::Setup(format!(
                "Required \"download_folder\" with value \"{}\" is not a folder.",
                download_folder.to_string_lossy()
            )));
        }

        Ok(Downloader {
            client: builder.build()?,
            downloads: vec![],
            parallel_requests: self.parallel_requests,
            retries: self.retries,
            download_folder: download_folder.to_owned(),
        })
    }
}
