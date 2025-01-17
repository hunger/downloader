// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (C) 2020 Tobias Hunger <tobias.hunger@gmail.com>
// Copyright (C) 2021 Phoenix IR <ayitsmephoenix@firemail.cc>

// Setup warnings/errors:
#![forbid(unsafe_code)]
#![deny(bare_trait_objects, unused_doc_comments, unused_import_braces)]
// Clippy:
#![warn(clippy::all, clippy::nursery, clippy::pedantic)]
#![allow(clippy::non_ascii_literal)]

use downloader::Downloader;
use std::env::temp_dir;

// Run example with: cargo run --example tui_basic --features tui
fn main() {
    let mut downloader = Downloader::builder()
        .download_folder(&temp_dir())
        .parallel_requests(1)
        .build()
        .unwrap();

    // Download with an explicit filename
    let dl = downloader::Download::new("https://example.org/")
        .file_name(std::path::Path::new("example.html"));

    // Download with an inferred filename
    let dl2 = downloader::Download::new(
        "https://cdimage.debian.org/debian-cd/12.8.0/i386/iso-cd/debian-12.8.0-i386-netinst.iso",
    );

    let result = downloader.download(&[dl, dl2]).unwrap();

    for r in result {
        match r {
            Err(e) => print!("Error occurred! {e}"),
            Ok(s) => print!("Success: {}", &s),
        };
    }
}
