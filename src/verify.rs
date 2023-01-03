// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (C) 2020 Tobias Hunger <tobias.hunger@gmail.com>

//! Verification callbacks

// cSpell: ignore hasher

// ----------------------------------------------------------------------
// - Types:
// ----------------------------------------------------------------------

/// A simplified progress callback passed to `Verify`. It only takes a progress
/// value, which is relative to the file length in bytes.
pub type SimpleProgress = dyn Fn(u64) + Sync;

/// A callback to used to verify the download.
pub type Verify =
    std::sync::Arc<dyn Fn(std::path::PathBuf, &SimpleProgress) -> Verification + Send + Sync>;

/// The possible states of file verification
#[derive(Debug, Eq, PartialEq)]
pub enum Verification {
    /// The file has not been verified at all.
    NotVerified,
    /// The file failed the verification process.
    Failed,
    /// The file passed the verification process.
    Ok,
}

impl std::fmt::Display for Verification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Self::NotVerified => "not verified",
                Self::Failed => "FAILED",
                Self::Ok => "Ok",
            }
        )
    }
}

// ----------------------------------------------------------------------
// - Noop:
// ----------------------------------------------------------------------

/// Do nothing to verify the download
#[must_use]
pub fn noop() -> crate::Verify {
    std::sync::Arc::new(|_: std::path::PathBuf, _: &crate::SimpleProgress| {
        Verification::NotVerified
    })
}

// ----------------------------------------------------------------------
// - SHA3:
// ----------------------------------------------------------------------

/// Make sure the downloaded file matches a provided hash using a provided Digest function
#[cfg(feature = "verify")]
#[must_use]
pub fn with_digest<D: digest::Digest>(hash: Vec<u8>) -> crate::Verify {
    use std::io::Read;

    std::sync::Arc::new(
        move |path: std::path::PathBuf, cb: &crate::SimpleProgress| {
            let mut hasher = D::new();

            if let Ok(file) = std::fs::OpenOptions::new().read(true).open(&path) {
                let mut reader = std::io::BufReader::with_capacity(1024 * 1024, file);
                let mut current = 0;

                let mut buffer = [0_u8; 1024 * 1024];
                while let Ok(n) = reader.read(&mut buffer[..]) {
                    if n == 0 {
                        break;
                    }

                    hasher.update(&buffer[..n]);

                    cb(current);
                    current += n as u64;
                }

                let result = hasher.finalize();

                if result.len() != hash.len() {
                    return Verification::Failed;
                }
                for i in 0..result.len() {
                    if result[i] != hash[i] {
                        return Verification::Failed;
                    }
                }
                return Verification::Ok;
            }

            Verification::Failed
        },
    )
}
