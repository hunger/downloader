# downloader

[![Crates.io](https://img.shields.io/crates/v/downloader.svg)](https://crates.io/crates/downloader)
[![Docs.rs](https://docs.rs/downloader/badge.svg)](https://docs.rs/downloader)
[![CI](https://github.com/hunger/downloader/workflows/Continuous%20Integration/badge.svg)](https://github.com/hunger/downloader/actions)
[![Coverage Status](https://coveralls.io/repos/github/hunger/downloader/badge.svg?branch=main)](https://coveralls.io/github/hunger/downloader?branch=main)
`downloader` is a crate to help with easly downloading of files from the
internet. It takes a simple simple and straightforward approach using a url
builder and fetcher.

It supports system proxy configuration, parallel downloads of different files,
validation of downloads via a callback, as well as files mirroring across different
machines.

Callbacks to provide progress information are supported as well.

## Installation and Usage

### Installation via Cargo

Add the following line into your `Cargo.toml` file to make `downloader` a
`[dependency]` of your crate:

`downloader = "<VERSION>"`

Alternatively you can run `cargo add downloader`. See crates.io for the latest
version of the package.

### Example

```rust
use downloader::downloader::Builder;
use downloader::Download;
use std::path::Path;
use std::time::Duration;

fn main() {
    let image = Download::new("https://example.com/example.png");
    // other downloads...
    // image.urls.push("https://example.com/example2.png");

    let mut dl = Builder::default()
        .connect_timeout(Duration::from_secs(4))
        .download_folder(Path::new("../res")) // or any arbitrary path
        .parallel_requests(8)
        .build()
        .unwrap();

    let response = dl.download(&[image]).unwrap(); // other error handling

    response.iter().for_each(|v| match v {
        Ok(v) => println!("Downloaded: {:?}", v),
        Err(e) => println!("Error: {:?}", e),
    })
}
```

### Features

- `tui` feature uses `indicatif` crate to provide a text ui for downloads
- `verify` feature enables (optional) verification of downloads using sha3 hashes

## License

Licensed under the GNU Lesser General Public License, Version 3.0 or later
([LICENSE-LGPLv3](LICENSE-LGPLv3.md) or <https://www.gnu.org/licenses/lgpl.md>

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, shall be licensed as LGPLv3 or later, without
any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md).
