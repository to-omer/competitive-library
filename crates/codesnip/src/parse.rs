use codesnip_core::{parse_file_recursive, Error};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use syn::{Item, Meta};

pub fn parse_files<P: AsRef<Path>>(targets: &[P], cfg: &[Meta]) -> Result<Vec<Item>, Error> {
    let mut items = Vec::new();
    let pb = ProgressBar::new(targets.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{prefix:>12.green} [{bar:57}] {pos}/{len}: {msg}")
            .progress_chars("=> "),
    );
    pb.set_prefix("Parsing");
    for target in targets.iter() {
        pb.set_message(&target.as_ref().display().to_string());
        items.append(&mut parse_file_recursive(target.as_ref().to_path_buf(), cfg)?.items);
        pb.inc(1);
    }
    pb.finish_and_clear();
    Ok(items)
}
