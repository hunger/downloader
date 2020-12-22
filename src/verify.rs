// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (C) 2020 Tobias Hunger <tobias.hunger@gmail.com>

//! Verification callbacks

// ----------------------------------------------------------------------
// - Noop:
// ----------------------------------------------------------------------

/// Do nothing to verify the download
#[must_use]
pub fn noop() -> crate::Verify {
    std::sync::Arc::new(|_: std::path::PathBuf, _: &crate::SimpleProgress| true)
}

// ----------------------------------------------------------------------
// - SHA3:
// ----------------------------------------------------------------------

/// Make sure the downloaded file matches a SHA3 hash
#[cfg(feature = "verify")]
#[must_use]
pub fn sha3_256(hash: Vec<u8>) -> crate::Verify {
    use sha3::Digest;
    use std::io::Read;

    std::sync::Arc::new(
        move |path: std::path::PathBuf, cb: &crate::SimpleProgress| {
            let mut hasher = sha3::Sha3_256::new();

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
                    return false;
                }
                for i in 0..result.len() {
                    if result[i] != hash[i] {
                        return false;
                    }
                }
                return true;
            }

            false
        },
    )
}
