// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (C) 2020 Tobias Hunger <tobias.hunger@gmail.com>

// Setup warnings/errors:
#![forbid(unsafe_code)]
#![deny(bare_trait_objects, unused_doc_comments, unused_import_braces)]
// Clippy:
#![warn(clippy::all, clippy::nursery, clippy::pedantic)]
#![allow(clippy::non_ascii_literal)]

use downloader::Downloader;

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
            if p.last_update.elapsed().as_millis() >= 1000 {
                println!(
                    "debian: {} of {:?}bytes. [{}]",
                    current, p.max_progress, p.message
                );
                p.last_update = std::time::Instant::now();
            }
        }
    }

    fn set_message(&self, message: &str) {
        println!("debian: Message changed to: {}", message);
    }

    fn done(&self) {
        let mut guard = self.private.lock().unwrap();
        *guard = None;
        println!("debian: [DONE]");
    }
}

fn main() {
    let mut downloader = Downloader::builder()
        .download_folder(std::path::Path::new("/tmp"))
        .parallel_requests(1)
        .build()
        .unwrap();

    let dl = downloader::Download::new(
        "https://cdimage.debian.org/debian-cd/current/amd64/iso-cd/debian-10.7.0-amd64-netinst.iso",
    );

    #[cfg(not(feature = "tui"))]
    let dl = dl.progress(SimpleReporter::create());

    #[cfg(feature = "verify")]
    let dl = dl.verify(downloader::verify::sha3_256(vec![
        0xb8, 0x96, 0xb7, 0xec, 0x34, 0xb9, 0x15, 0x64, 0x29, 0xe0, 0x19, 0xee, 0x33, 0xd8, 0x33,
        0x10, 0xa4, 0x47, 0xb4, 0x6d, 0x35, 0xb2, 0x52, 0x1d, 0x8e, 0x7c, 0x22, 0xdc, 0xb5, 0x50,
        0x73, 0xf9,
    ]));

    let result = downloader.download(&[dl]).unwrap();

    for r in result {
        match r {
            Err(e) => println!("Error: {}", e.to_string()),
            Ok(s) => println!("Success: {}", &s),
        };
    }
}
