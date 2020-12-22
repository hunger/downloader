// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (C) 2020 Tobias Hunger <tobias.hunger@gmail.com>

//! This crate provides a simple way to download files via HTTP/HTTPS.
//!
//! It tries to make it very simple to just specify a couple of
//! URLs and then go and download all of the files.
//!
//! It supports system proxy configuration, parallel downloads of different files,
//! validation of downloads via a callback, as well as files being mirrored on
//! different machines.
//!
//! Callbacks to provide progress information are supported as well.

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

pub mod backend;
pub mod download;
pub mod downloader;
pub mod progress;
pub mod verify;

pub use download::{Download, Progress, SimpleProgress, Verify};
pub use downloader::Downloader;

// ----------------------------------------------------------------------
// - Error:
// ----------------------------------------------------------------------

/// Possible `Error`s that can occurred during normal operation.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// The Setup is incomplete or bogus.
    #[error("Setup error: {0}")]
    Setup(String),
    /// A Definition of a Download is incomplete
    #[error("Download definition: {0}")]
    DownloadDefinition(String),
}

/// `Result` type for the `gng_shared` library
pub type Result<T> = std::result::Result<T, Error>;

// ----------------------------------------------------------------------
// - DownloadResult:
// ----------------------------------------------------------------------

/// The result of a `Download`
pub struct DownloadResult {
    /// A list of attempted downloads with URL and status code.
    pub status: Vec<(String, u16)>,
    /// The path this URL has been downloaded to.
    pub file_name: std::path::PathBuf,
    /// Verification was successful?
    pub verified: bool,
}

impl DownloadResult {
    /// Returns whether this the downloaded file is ready for use.
    #[must_use]
    pub fn was_success(&self) -> bool {
        self.status.last().unwrap_or(&(String::from(""), 0)).1 == 200 && self.verified
    }

    /// Returns whether this the file has been downloaded successfully.
    #[must_use]
    pub fn was_downloaded(&self) -> bool {
        self.status.last().unwrap_or(&(String::from(""), 0)).1 == 200
    }

    /// Returns whether this verification was a success.
    #[must_use]
    pub const fn was_verified(&self) -> bool {
        self.verified
    }
}
