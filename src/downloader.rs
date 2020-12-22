// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (C) 2020 Tobias Hunger <tobias.hunger@gmail.com>

//! The `Downloader` struct

use crate::{Download, DownloadResult, Error, Result};

use crate::progress::Factory;

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
    /// Create a `Builder` for `Downloader`
    #[must_use]
    pub fn builder() -> Builder {
        let download_folder =
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(""));
        let download_folder = if download_folder.to_string_lossy().is_empty() {
            std::path::PathBuf::from(
                std::env::var_os("HOME").unwrap_or_else(|| std::ffi::OsString::from("/")),
            )
        } else {
            download_folder
        };

        Builder {
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

        #[cfg(feature = "tui")]
        let factory = crate::progress::Tui::default();
        #[cfg(not(feature = "tui"))]
        let factory = crate::progress::Noop::default();

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

            if d.progress.is_none() {
                d.progress = Some(factory.create_reporter());
            }
        }

        Ok(crate::backend::run(
            &mut self.client,
            to_process,
            self.retries,
            self.parallel_requests,
            &move || {
                factory.join();
            },
        ))
    }
}

// ----------------------------------------------------------------------
// - Builder:
// ----------------------------------------------------------------------

/// A builder for `Downloader`
pub struct Builder {
    user_agent: String,
    connect_timeout: std::time::Duration,
    timeout: std::time::Duration,
    parallel_requests: u16,
    retries: u16,
    download_folder: std::path::PathBuf,
}

impl Builder {
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
    pub fn build(&mut self) -> crate::Result<Downloader> {
        let builder = reqwest::Client::builder()
            .user_agent(self.user_agent.clone())
            .connect_timeout(self.connect_timeout)
            .timeout(self.timeout);

        let download_folder = &self.download_folder;
        if download_folder.to_string_lossy().is_empty() {
            return Err(crate::Error::Setup(
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
