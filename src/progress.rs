use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{sync::Arc, time::Duration};

pub fn build_total_progress_bar() -> (Arc<MultiProgress>, Arc<ProgressBar>) {
    let multi = Arc::new(MultiProgress::new());
    let total_bar = Arc::new(multi.add(ProgressBar::new(0)));

    total_bar.enable_steady_tick(Duration::from_millis(120));
    total_bar.set_style(
        ProgressStyle::default_bar()
            .progress_chars("-> ")
            .template("{spinner} Fetching data · {msg:.blue}")
            .unwrap(),
    );

    (multi, total_bar)
}

pub fn prepare_total_bar_for_download(bar: &ProgressBar, job_count: usize) {
    bar.disable_steady_tick();
    bar.set_length(job_count as u64);
    bar.set_style(
        ProgressStyle::default_bar()
            .progress_chars("-> ")
            .template("[{bar:30.blue}] {pos}/{len} ({percent}%) · {msg:.green} · {prefix:.red}")
            .unwrap(),
    );
    bar.set_message("Downloading");
}
