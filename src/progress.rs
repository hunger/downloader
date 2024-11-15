// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (C) 2020 Tobias Hunger <tobias.hunger@gmail.com>

//! Progress reporting code

// ----------------------------------------------------------------------
// - Types:
// ----------------------------------------------------------------------

/// A Progress reporter to use for the `Download`
pub type Progress = std::sync::Arc<dyn Reporter>;

// ----------------------------------------------------------------------
// - Traits:
// ----------------------------------------------------------------------

/// An interface for `ProgressReporter`s
pub trait Reporter: Send + Sync {
    /// Setup a TUI element for the next progress
    fn setup(&self, max_progress: Option<u64>, message: &str);
    /// Report progress
    fn progress(&self, current: u64);
    /// Report progress
    fn set_message(&self, message: &str);
    /// Finish up after progress reporting is done
    fn done(&self);
}

/// A `Factory` used to create `Reporter` when a Download does
/// not come with a defined way to report progress already.
pub trait Factory {
    /// Create an `Reporter`
    fn create_reporter(&self) -> crate::Progress;
}

// ----------------------------------------------------------------------
// - Noop:
// ----------------------------------------------------------------------

/// Do not print anything
#[derive(Default)]
pub struct Noop {}

impl Reporter for Noop {
    fn setup(&self, _: Option<u64>, _: &str) {}

    fn progress(&self, _: u64) {}

    fn set_message(&self, _: &str) {}

    fn done(&self) {}
}

impl Noop {
    /// Create a `Noop` `Reporter`.
    #[must_use]
    pub fn create() -> crate::Progress {
        std::sync::Arc::new(Self {})
    }
}

impl Factory for Noop {
    fn create_reporter(&self) -> crate::Progress {
        Self::create()
    }
}

// ----------------------------------------------------------------------
// - TUI:
// ----------------------------------------------------------------------

#[cfg(feature = "tui")]
mod tui {
    /// Manage multiple progress reporters combined into one set of progress bars
    pub struct Tui {
        progress_group: indicatif::MultiProgress,
    }

    impl Default for Tui {
        fn default() -> Self {
            Self {
                progress_group: indicatif::MultiProgress::with_draw_target(
                    indicatif::ProgressDrawTarget::stderr_with_hz(4),
                ),
            }
        }
    }

    impl super::Factory for Tui {
        /// Create a `Reporter` connected to this set of UI primitives.
        fn create_reporter(&self) -> crate::Progress {
            std::sync::Arc::new(TuiBar {
                progress_bar: std::sync::Mutex::new(
                    self.progress_group.add(indicatif::ProgressBar::new(1)),
                ),
            })
        }
    }

    struct TuiBar {
        progress_bar: std::sync::Mutex<indicatif::ProgressBar>,
    }

    impl super::Reporter for TuiBar {
        fn setup(&self, max_progress: Option<u64>, message: &str) {
            let lock = self.progress_bar.lock().unwrap();
            if let Some(t) = max_progress {
                lock.set_length(t);
                lock.set_style(indicatif::ProgressStyle::default_bar()
                .template("[{bar:20.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta}) - {msg}")
                .unwrap()
                .progress_chars("#- "));
                lock.set_message(String::from(message));
                lock.reset_eta();
            } else {
                lock.set_style(
                    indicatif::ProgressStyle::default_spinner()
                        // For more spinners check out the cli-spinners project:
                        // https://github.com/sindresorhus/cli-spinners/blob/master/spinners.json
                        .tick_strings(&[
                            "▹▹▹▹▹",
                            "▸▹▹▹▹",
                            "▹▸▹▹▹",
                            "▹▹▸▹▹",
                            "▹▹▹▸▹",
                            "▹▹▹▹▸",
                            "▪▪▪▪▪",
                        ])
                        .template("{spinner:.blue} {msg}")
                        .unwrap()
                );
                lock.set_message(String::from(message))
            };
        }

        fn progress(&self, current: u64) {
            let lock = self.progress_bar.lock().unwrap();
            lock.set_position(current);
        }

        fn set_message(&self, message: &str) {
            let lock = self.progress_bar.lock().unwrap();
            lock.set_message(String::from(message));
        }

        fn done(&self) {
            let lock = self.progress_bar.lock().unwrap();
            lock.finish();
        }
    }
}

#[cfg(feature = "tui")]
pub use tui::Tui;
