use indicatif::{ProgressBar, ProgressStyle};

pub trait ProgressReporter {
    fn set_length(&mut self, total: u64);
    fn set_message(&mut self, message: &str);
    fn inc(&mut self, delta: u64);
    fn finish_with_message(&mut self, message: &str);
    fn abandon(&mut self, message: &str);
}

#[derive(Default)]
pub struct IndicatifReporter {
    bar: Option<ProgressBar>,
}

impl IndicatifReporter {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ProgressReporter for IndicatifReporter {
    fn set_length(&mut self, total: u64) {
        let bar = ProgressBar::new(total);
        bar.set_style(
            ProgressStyle::with_template(
                "[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
            )
            .unwrap()
            .progress_chars("=>-"),
        );
        self.bar = Some(bar);
    }

    fn set_message(&mut self, message: &str) {
        if let Some(bar) = &self.bar {
            bar.set_message(message.to_owned());
        }
    }

    fn inc(&mut self, delta: u64) {
        if let Some(bar) = &self.bar {
            bar.inc(delta);
        }
    }

    fn finish_with_message(&mut self, message: &str) {
        if let Some(bar) = self.bar.take() {
            bar.finish_with_message(message.to_owned());
        }
    }

    fn abandon(&mut self, message: &str) {
        if let Some(bar) = self.bar.take() {
            bar.abandon_with_message(message.to_owned());
        }
    }
}

pub struct SilentReporter;

impl ProgressReporter for SilentReporter {
    fn set_length(&mut self, _total: u64) {}
    fn set_message(&mut self, _message: &str) {}
    fn inc(&mut self, _delta: u64) {}
    fn finish_with_message(&mut self, _message: &str) {}
    fn abandon(&mut self, _message: &str) {}
}
