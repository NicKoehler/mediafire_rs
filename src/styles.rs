use indicatif::ProgressStyle;

#[derive(Clone)]
pub struct ProgressStyles {
    pub normal: ProgressStyle,
    pub error: ProgressStyle,
    pub download: ProgressStyle,
}

impl ProgressStyles {
    pub fn new() -> Self {
        Self {
            normal: ProgressStyle::default_bar()
                .progress_chars("-> ")
                .template("[{bar:30.green}] · {msg} · {prefix:.blue}")
                .unwrap(),

            error: ProgressStyle::default_bar()
                .progress_chars("-> ")
                .template("[{bar:30.green}] · {msg} · {prefix:.red}")
                .unwrap(),

            download: ProgressStyle::default_bar()
                .progress_chars("-> ")
                .template(
                    "[{bar:30.green}] · {msg} · {percent}% \
                     ({bytes:.magenta}/{total_bytes:.magenta}) · {prefix:.blue}",
                )
                .unwrap(),
        }
    }
}
