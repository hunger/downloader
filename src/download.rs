// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (C) 2020 Tobias Hunger <tobias.hunger@gmail.com>

//! The actual download code

use crate::{Download, DownloadResult};

use futures::stream::{self, StreamExt};
use rand::seq::SliceRandom;

use std::io::Write;

fn select_url(urls: &[String]) -> String {
    assert!(!urls.is_empty());
    urls.choose(&mut rand::thread_rng()).unwrap().to_owned()
}

async fn download_url(
    client: &reqwest::Client,
    url: &str,
    writer: &mut std::io::BufWriter<std::fs::File>,
    callback: &crate::ProgressCallback,
    retry: u16,
    retries: u16,
) -> u16 {
    if let Ok(mut response) = client.get(url).send().await {
        let total = response.content_length();
        let mut current: u64 = 0;

        callback(retry, retries, current, total); // Start progress reporting with 0

        while let Some(bytes) = response.chunk().await.unwrap_or(None) {
            if writer.write_all(&bytes).is_err() {}

            current += bytes.len() as u64;
            callback(retry, retries, current, total);
        }

        callback(retry, retries, current, total);

        response.status().as_u16()
    } else {
        reqwest::StatusCode::BAD_REQUEST.as_u16()
    }
}

async fn download(client: reqwest::Client, download: Download, retries: u16) -> DownloadResult {
    let mut status = Vec::new();

    let mut urls = download.urls.clone();
    assert!(!urls.is_empty());

    let file_name = download.file_name;

    if let Ok(file) = std::fs::File::create(&file_name) {
        let mut writer = std::io::BufWriter::new(file);

        for retry in 1..=retries {
            let url = select_url(&urls);

            let s = reqwest::StatusCode::from_u16(
                download_url(
                    &client,
                    &url,
                    &mut writer,
                    &download.callback,
                    retry,
                    retries,
                )
                .await,
            )
            .unwrap_or(reqwest::StatusCode::BAD_REQUEST);

            status.push((url.clone(), s.as_u16()));

            if s.is_server_error() {
                urls = urls
                    .iter()
                    .filter_map(|u| if u == &url { Some(u.to_owned()) } else { None })
                    .collect();
                if urls.is_empty() {
                    break;
                }
            }

            if s.is_success() {
                break;
            }
        }
    }

    DownloadResult { status, file_name }
}

/// Run the provided list of `downloads`, using the provided `client`
pub(crate) fn run(
    client: &mut reqwest::Client,
    downloads: Vec<Download>,
    retries: u16,
    parallel_requests: u16,
) -> Vec<DownloadResult> {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let cl = client.clone();

    let result = rt.spawn(async move {
        stream::iter(downloads)
            .map(move |d| download(cl.clone(), d, retries))
            .buffer_unordered(parallel_requests as usize)
            .collect::<Vec<DownloadResult>>()
            .await
    });

    rt.block_on(result).unwrap()
}
