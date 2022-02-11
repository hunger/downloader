// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (C) 2020 Tobias Hunger <tobias.hunger@gmail.com>

// Setup warnings/errors:
#![forbid(unsafe_code)]
#![deny(bare_trait_objects, unused_doc_comments, unused_import_braces)]
// Clippy:
#![warn(clippy::all, clippy::nursery, clippy::pedantic)]
#![allow(clippy::non_ascii_literal)]

use downloader::{Downloader, MirrorContext};

// Define a custom progress reporter:
struct SimpleReporterPrivate {
    last_update: std::time::Instant,
    max_progress: Option<u64>,
    message: String,
}
struct SimpleReporter {
    private: std::sync::Mutex<Option<SimpleReporterPrivate>>,
}

impl SimpleReporter {
    #[cfg(not(feature = "tui"))]
    fn create() -> std::sync::Arc<Self> {
        std::sync::Arc::new(Self {
            private: std::sync::Mutex::new(None),
        })
    }
}

impl downloader::progress::Reporter for SimpleReporter {
    fn setup(&self, max_progress: Option<u64>, message: &str) {
        let private = SimpleReporterPrivate {
            last_update: std::time::Instant::now(),
            max_progress,
            message: message.to_owned(),
        };

        let mut guard = self.private.lock().unwrap();
        *guard = Some(private);
    }

    fn progress(&self, current: u64) {
        if let Some(p) = self.private.lock().unwrap().as_mut() {
            let max_bytes = match p.max_progress {
                Some(bytes) => format!("{:?}", bytes),
                None => "{unknown}".to_owned(),
            };
            if p.last_update.elapsed().as_millis() >= 1000 {
                println!(
                    "test file: {} of {} bytes. [{}]",
                    current, max_bytes, p.message
                );
                p.last_update = std::time::Instant::now();
            }
        }
    }

    fn set_message(&self, message: &str) {
        println!("test file: Message changed to: {}", message);
    }

    fn done(&self) {
        let mut guard = self.private.lock().unwrap();
        *guard = None;
        println!("test file: [DONE]");
    }
}

fn main() {
    let mut downloader = Downloader::builder()
        .download_folder(std::path::Path::new("/tmp"))
        .parallel_requests(1)
        .build()
        .unwrap();

    let dl = downloader::Download::new("https://speed.hetzner.de/100MB.bin");

    #[cfg(not(feature = "tui"))]
    let dl = dl.progress(SimpleReporter::create());

    #[cfg(feature = "verify")]
    let dl = {
        use downloader::verify;
        fn decode_hex(s: &str) -> Result<Vec<u8>, std::num::ParseIntError> {
            (0..s.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
                .collect()
        }
        dl.verify(verify::with_digest::<sha3::Sha3_256>(
            decode_hex("2197e485d463ac2b868e87f0d4547b4223ff5220a0694af2593cbe7c796f7fd6").unwrap(),
        ))
    };

    let result = downloader.download(&[dl]).unwrap();

    for r in result {
        match r {
            Err(e) => println!("Error: {}", e.to_string()),
            Ok(s) => println!("Success: {}", &s),
        };
    }

    let debian_mirrors = MirrorContext::from_urls(&[
        "http://ftp.au.debian.org/debian/",
        "http://ftp.ca.debian.org/debian/",
        "http://ftp.cz.debian.org/debian/",
        "http://ftp.us.debian.org/debian/",
    ]).unwrap();

    let dl = downloader::Download::new_mirrored(debian_mirrors.urls_for_file("README.html").unwrap().as_slice());

    let result = downloader.download(&[dl]).unwrap();

    for r in result {
        match r {
            Err(e) => println!("Error: {}", e.to_string()),
            Ok(s) => println!("Success: {}", &s),
        };
    }
}
