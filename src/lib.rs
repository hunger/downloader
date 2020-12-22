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

pub use download::Download;
pub use downloader::Downloader;
pub use progress::Progress;
pub use verify::{SimpleProgress, Verification, Verify};

// ----------------------------------------------------------------------
// - Error:
// ----------------------------------------------------------------------

/// Possible `Error`s that can occurred during normal operation.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// The Setup is incomplete or bogus.
    #[error("Setup error: {0}")]
    Setup(String),
    /// A Definition of a `Download` is incomplete
    #[error("Download definition: {0}")]
    DownloadDefinition(String),
    /// Writing into a file failed during download.
    #[error("File creation failed: {0}")]
    File(DownloadSummary),
    /// A download failed
    #[error("Download failed for {0}")]
    Download(DownloadSummary),
    /// Download file verification failed.
    #[error("Verification failed for {0}")]
    Verification(DownloadSummary),
}

/// `Result` type for the `gng_shared` library
pub type Result<T> = std::result::Result<T, Error>;

// ----------------------------------------------------------------------
// - DownloadSummary:
// ----------------------------------------------------------------------

/// The result of a `Download`
pub struct DownloadSummary {
    /// A list of attempted downloads with URL and status code.
    pub status: Vec<(String, u16)>,
    /// The path this URL has been downloaded to.
    pub file_name: std::path::PathBuf,
    /// File verification status
    pub verified: Verification,
}

fn to_fmt(f: &mut std::fmt::Formatter<'_>, summary: &DownloadSummary) -> std::fmt::Result {
    writeln!(
        f,
        "{}: (verification: {}):",
        summary.file_name.to_string_lossy(),
        match summary.verified {
            Verification::NotVerified => "unverified",
            Verification::Failed => "FAILED",
            Verification::Ok => "Ok",
        },
    )?;
    for i in 0..summary.status.len() {
        writeln!(
            f,
            "  {}: {} with status {}",
            i + 1,
            summary.status[i].0,
            summary.status[i].1
        )?;
    }
    Ok(())
}

impl std::fmt::Display for DownloadSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        to_fmt(f, self)
    }
}

impl std::fmt::Debug for DownloadSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        to_fmt(f, self)
    }
}
