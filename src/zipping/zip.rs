use crate::Result;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use path_slash::PathExt;
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::ZipWriter;

pub fn zip_folder<F: std::io::Write + std::io::Seek>(
    zip: &mut ZipWriter<F>,
    src_dir: impl AsRef<Path>,
    zip_options: FileOptions,
) -> Result<()> {
    let src_dir = src_dir.as_ref();

    let walkdir = WalkDir::new(src_dir);
    let it = walkdir.into_iter();

    let mut buffer = Vec::new();
    for entry in it {
        let entry = entry?;
        let path = entry.path();

        let name = path
            .strip_prefix(src_dir.parent().expect("Could not get a parent folder"))
            .unwrap();
        let name = name
            .to_str()
            .expect("Could not convert file name to Unicode");

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            let mut file_options = zip_options.clone();
            // Get and Set permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;

                let unix_mode: u32 = std::fs::metadata(path)?.permissions().mode();
                file_options = file_options.unix_permissions(unix_mode);
            }

            zip.start_file(name, file_options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&*buffer)?;
            buffer.clear();
        } else if name.len() != 0 {
            // zip requires that folder separator is /, even on windows
            let directory_name = if cfg!(windows) {
                Path::new(name).to_slash().unwrap_or(name.to_owned())
            } else {
                name.to_owned()
            };
            zip.add_directory(directory_name, zip_options)?;
        }
    }

    Ok(())
}

pub fn zip_file<F: std::io::Write + std::io::Seek>(
    zip: &mut ZipWriter<F>,
    file: impl AsRef<Path>,
    mut zip_options: FileOptions,
) -> Result<()> {
    let file = file.as_ref();
    let name = file
        .file_name()
        .expect("Could not get file name")
        .to_str()
        .expect("Could not convert file name to Unicode");

    // Get and Set permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let unix_mode: u32 = std::fs::metadata(file)?.permissions().mode();
        zip_options = zip_options.unix_permissions(unix_mode);
    }

    zip.start_file(name, zip_options)?;

    let mut f = File::open(file)?;
    let mut buffer = Vec::new();

    f.read_to_end(&mut buffer)?;
    zip.write_all(buffer.as_slice())?;
    buffer.clear();

    Ok(())
}
