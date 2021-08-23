use futures::{stream, StreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use crate::Result;

#[derive(Debug, Clone)]
pub struct FileToUnzip {
    archive: PathBuf,
    output: PathBuf,
}

impl FileToUnzip {
    pub fn new(archive: impl Into<PathBuf>, output: impl Into<PathBuf>) -> Self {
        Self {
            archive: archive.into(),
            output: output.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FilesToUnzip {
    files: Vec<FileToUnzip>,
}

impl FilesToUnzip {
    pub fn new() -> Self {
        Self { files: vec![] }
    }

    pub fn add(self, file_to_unzip: FileToUnzip) -> Self {
        let mut files = self.files.clone();
        files.push(file_to_unzip);
        Self { files }
    }

    pub fn maybe_add(self, file_to_unzip: Option<FileToUnzip>) -> Self {
        if let Some(file_to_unzip) = file_to_unzip {
            self.add(file_to_unzip)
        } else {
            self
        }
    }

    pub fn extend(self, files_to_unzip: Self) -> Self {
        let mut files = self.files.clone();
        files.extend(files_to_unzip.files);
        Self { files }
    }

    pub async fn unzip(self) -> Result<()> {
        // Set up a new multi-progress bar.
        // The bar is stored in an `Arc` to facilitate sharing between threads.
        let multibar = std::sync::Arc::new(indicatif::MultiProgress::new());
        // Add an overall progress indicator to the multibar.
        // It has as many steps as the download_links Vector and will increment on completion of each task.
        let main_pb = std::sync::Arc::new(
            multibar
                .clone()
                .add(indicatif::ProgressBar::new(self.files.len() as u64)),
        );

        main_pb.set_style(
            indicatif::ProgressStyle::default_bar().template("{msg} {bar:10} {pos}/{len}"),
        );
        main_pb.set_message("total  ");

        // Make the main progress bar render immediately rather than waiting for the
        // first task to finish.
        main_pb.tick();

        // Convert download_links Vector into stream
        // This is basically a async compatible iterator
        let stream = stream::iter(&self.files);

        // Set up a future to iterate over tasks and run up to 2 at a time.
        let tasks = stream
            .enumerate()
            .for_each_concurrent(Some(2), |(_i, file_to_unzip)| {
                // Clone multibar and main_pb.  We will move the clones into each task.
                let multibar = multibar.clone();
                let main_pb = main_pb.clone();
                let file_to_unzip = file_to_unzip.clone();
                async move {
                    // Spawn a new tokio task for the current file to unzip
                    // We need to hand over the multibar, so the ProgressBar for the task can be added
                    let _task = tokio::task::spawn(futures::future::lazy(|_| {
                        unzip_task(file_to_unzip, multibar)
                    }))
                    .await;

                    // Increase main ProgressBar by 1
                    main_pb.inc(1);
                }
            });

        // Set up a future to manage rendering of the multiple progress bars.
        let multibar = {
            // Create a clone of the multibar, which we will move into the task.
            let multibar = multibar.clone();

            // multibar.join() is *not* async and will block until all the progress
            // bars are done, therefore we must spawn it on a separate scheduler
            // on which blocking behavior is allowed.
            tokio::task::spawn_blocking(move || multibar.join())
        };

        // Wait for the tasks to finish.
        tasks.await;

        // Change the message on the overall progress indicator.
        main_pb.finish_with_message("done");

        // Wait for the progress bars to finish rendering.
        // The first ? unwraps the outer join() in which we are waiting for the
        // future spawned by tokio::task::spawn_blocking to finishe.
        // The second ? unwraps the inner multibar.join().
        Ok(multibar.await??)
    }
}

pub fn unzip_task(file_to_unzip: FileToUnzip, multibar: Arc<MultiProgress>) -> Result<()> {
    let file = std::fs::File::open(&file_to_unzip.archive).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();

    // Create the ProgressBar with the aquired size from before
    // and add it to the multibar
    let progress_bar = multibar.add(ProgressBar::new(archive.len() as u64));

    // Set Style to the ProgressBar
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{bar:40.cyan/blue}] {percent}% - {msg}")
            .progress_chars("#>-"),
    );

    // Set the filename as message part of the progress bar
    progress_bar.set_message(
        file_to_unzip
            .archive
            .clone()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string(),
    );

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();

        let output_path = match file.enclosed_name() {
            Some(path) => file_to_unzip.output.join(path),
            None => continue,
        };

        if (&*file.name()).ends_with('/') {
            std::fs::create_dir_all(&output_path).unwrap();
        } else {
            if let Some(p) = output_path.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(&p).unwrap();
                }
            }
            let mut outfile = std::fs::File::create(&output_path).unwrap();
            std::io::copy(&mut file, &mut outfile).unwrap();
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                std::fs::set_permissions(&output_path, fs::Permissions::from_mode(mode)).unwrap();
            }
        }
        progress_bar.inc(1)
    }

    // Finish the progress bar to prevent glitches
    progress_bar.finish();

    Ok(())
}
