// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (C) 2020 Tobias Hunger <tobias.hunger@gmail.com>

// Setup warnings/errors:
#![forbid(unsafe_code)]
#![deny(bare_trait_objects, unused_doc_comments, unused_import_braces)]
// Clippy:
#![warn(clippy::all, clippy::nursery, clippy::pedantic)]
#![allow(clippy::non_ascii_literal)]

use downloader::Downloader;

fn main() {
    let mut downloader = Downloader::builder()
        .download_folder(std::path::Path::new("/tmp"))
        .parallel_requests(1)
        .build()
        .unwrap();

    downloader.queue(downloader::Download::new(
        "https://cdimage.debian.org/debian-cd/current/amd64/iso-cd/debian-10.7.0-amd64-netinst.iso",
    )
    .callback(Box::new(|r: u16, rm: u16, b: u64, bm: Option<u64>| {
        println!("debian: {}/{} retries, {} of {:?}bytes.", r, rm, b, bm)
    })));

    downloader.queue(downloader::Download::new(
        "https://download.fedoraproject.org/pub/fedora/linux/releases/33/Server/x86_64/iso/Fedora-Server-netinst-x86_64-33-1.2.iso",
    )
    .callback(Box::new(|r: u16, rm: u16, b: u64, bm: Option<u64>| {
        println!("fedora: {}/{} retries, {} of {:?}bytes.", r, rm, b, bm)
    })));

    let result = downloader.download().unwrap();

    for r in result {
        println!(
            "Download: {}: {} ({}).",
            r.file_name.to_string_lossy(),
            r.status.last().unwrap_or(&(String::new(), 0)).1,
            if r.is_success() { "SUCCESS" } else { "FAILED" },
        )
    }
}
