// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (C) 2020 Tobias Hunger <tobias.hunger@gmail.com>

// Setup warnings/errors:
#![forbid(unsafe_code)]
#![deny(bare_trait_objects, unused_doc_comments, unused_import_braces)]
// Clippy:
#![warn(clippy::all, clippy::nursery, clippy::pedantic)]
#![allow(clippy::non_ascii_literal)]

use downloader::Downloader;

// Run example with: cargo run --example tui_basic --features tui
fn main() {
    let mut downloader = Downloader::builder()
        .download_folder(std::path::Path::new("/tmp"))
        .parallel_requests(1)
        .build()
        .unwrap();

    let dl = downloader::Download::new("https://example.org/")
        .file_name(std::path::Path::new("example.html"));

    let result = downloader.download(&[dl]).unwrap();

    for r in result {
        match r {
            Err(e) => print!("Error occured! {}", e.to_string()),
            Ok(s) => print!("Success: {}", &s),
        };
    }
}
