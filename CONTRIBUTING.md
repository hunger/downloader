# Contribution guidelines

First off, thank you for considering contributing to downloader.

If your contribution is not straightforward, please first discuss the change you
wish to make by creating a new issue before making the change.

## Reporting issues

Before reporting an issue on the
[issue tracker](https://github.com/MyUsername/downloader/issues),
please check that it has not already been reported by searching for some related
keywords.

## Pull requests

Try to do one pull request per change.

## Developing

### Set up

This is no different than other Rust projects.

```shell
git clone https://github.com/MyUsername/downloader
cd downloader
cargo build
```

### Useful Commands

- Build and run release version:

  ```shell
  cargo build --release && cargo run --release
  ```

- Run Clippy:

  ```shell
  cargo clippy --all
  ```

- Run all tests:

  ```shell
  cargo test --all
  ```

- Check to see if there are code formatting issues

  ```shell
  cargo fmt --all -- --check
  ```

- Format the code in the project

  ```shell
  cargo fmt --all
  ```
