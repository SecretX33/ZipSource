mod chars;
#[macro_use]
mod log_macros;

use std::{env, fs, io, thread};
use std::collections::HashSet;
use std::fs::{File};
use std::io::{Write};
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, mpsc};
use std::sync::atomic::{AtomicU32, Ordering};
use size::{Size, Style};
use threadpool::ThreadPool;
use walkdir::{WalkDir};
use zip::{CompressionMethod, ZipWriter};
use zip::result::ZipResult;
use zip::write::FileOptions;
use crate::chars::unescape;

/// After some experimentation, on my machine this number was the fastest, no deep reason why
/// it is this, though.
const MAX_CONCURRENT_THREADS: usize = 9;

fn main() {
    log!("Started collecting files");

    let settings = get_settings();
    debug_log!("Settings: {:?}", settings);

    let all_files = list_files_recursive(&settings.base_dir);
    let all_files_path = all_files.iter().map(|p| p.as_path()).collect();
    log!("Found a total of {} files in the folder.", all_files.len());

    let valid_files = filter_valid_files(&settings.base_dir, all_files_path);
    log!("There are {} files to be zipped (out of {}, which means {} files were ignored)", valid_files.len(), all_files.len(), all_files.len() - valid_files.len());

    zip_files(&settings.zip_path, &settings.base_dir, valid_files).expect("Creation of zip failed");

    let file_size = pretty_file_size(&settings.zip_path).expect("Could not read zip file info");
    log!("Zip was successfully created (file size is {})!", file_size.replace("iB", "B"));
}

fn get_settings() -> Settings {
    let args: Vec<_> = env::args().into_iter().skip(1).collect();

    let base_dir = args.first().map(PathBuf::from)
        .or(env::current_dir().ok())
        .expect("Could not determine base directory setting");

    let zip_name = args.get(1).cloned().or(
        base_dir.file_name()
            .and_then(|name| name.to_str())
            .map(|name| format!("{name} (Source Code).zip"))
    ).expect("Could not determine zip name setting");

    let zip_path = base_dir.join(&zip_name);

    Settings {
        base_dir,
        zip_path,
    }
}

fn list_files_recursive(directory: &Path) -> HashSet<PathBuf> {
    WalkDir::new(directory).into_iter()
        .filter_map(|file| file.ok())
        .filter(|file| file.file_type().is_file())
        .map(|file| file.into_path())
        .collect()
}

fn filter_valid_files<'a>(current_dir: &Path, all_files: HashSet<&'a Path>) -> HashSet<&'a Path> {
    let all_files_vector: Vec<_> = all_files.iter().copied().collect();
    let files_chunked: Vec<HashSet<_>> = all_files_vector.chunks(50)
        .map(|x| x.iter().map(|p| p.to_path_buf()).collect())
        .collect();

    let chunks_amount = files_chunked.len();
    let pool_size = calculate_ideal_parallelism(chunks_amount);
    debug_log!("There are {} chunks of files to be processed (thread pool size: {})", chunks_amount, pool_size);

    let pool = ThreadPool::new(pool_size);
    let count = Arc::from(AtomicU32::new(0));
    let (tx, rx) = mpsc::channel();

    for files in files_chunked {
        let tx = tx.clone();
        let count = Arc::clone(&count);
        let current_dir = current_dir.to_path_buf();

        pool.execute(move|| {
            let files_path: HashSet<&Path> = files.iter().map(|p| p.as_path()).collect();
            let mut command = build_check_ignore_command(&current_dir, &files_path);
            let ignored_files = run_ignored_files_command(&mut command);
            debug_log!("[{}] Chunk size: {}, ignored files size: {}", count.fetch_add(1, Ordering::Relaxed), files.len(), ignored_files.len());

            tx.send(ignored_files).expect("channel will be there waiting for the pool");
        });
    }

    // Await for all jobs to finish
    pool.join();

    // Collect their results
    let ignored_files: HashSet<_> = rx.iter().take(chunks_amount).flatten().collect();
    let ignored_files: HashSet<&Path> = ignored_files.iter().map(|e| e.as_path()).collect();

    debug_log!("\nall_files: {:?}\nignored_files: {:?}\n",
        all_files.iter().take(6).collect::<Vec<_>>(),
        ignored_files.iter().take(6).collect::<Vec<_>>()
    );

    let valid_files: HashSet<&Path> = all_files_vector.iter()
        .copied()
        .filter(|&path| !ignored_files.contains(path) && !is_excluded_specially(current_dir, path))
        .collect();

    debug_log!(
        "ignored_files: {}, all_files_vector size: {}, valid_files size: {}",
        ignored_files.len(),
        all_files_vector.len(),
        valid_files.len()
    );
    valid_files
}

fn build_check_ignore_command(current_dir: &Path, files: &HashSet<&Path>) -> Command {
    let path_list: Vec<&str> = files.iter()
        .filter_map(|path| path.to_str())
        .collect();

    let (cmd, first_arg) = if cfg!(target_os = "windows") { ("cmd", "/C") } else { ("sh", "-c") };
    let args = [[first_arg, "git", "check-ignore"].as_slice(), path_list.as_slice()].concat();

    let mut command = Command::new(cmd);
    command.current_dir(current_dir).args(args);
    command
}

fn run_ignored_files_command(command: &mut Command) -> HashSet<PathBuf> {
    let output = command.output().expect("Failed to execute process");

    String::from_utf8_lossy(&output.stdout).trim()
        .split("\n")
        .map(|e| PathBuf::from(unescape(&e.to_string()).expect("Invalid path was found")))
        .collect()
}

fn is_excluded_specially(current_dir: &Path, file_to_check: &Path) -> bool {
    let paths_to_exclude = [current_dir.join(".git")];

    paths_to_exclude.iter().any(|p| file_to_check.starts_with(p))
}

fn zip_files(zip_path: &Path, base_dir: &Path, files: HashSet<&Path>) -> ZipResult<()> {
    if zip_path.exists() {
        let filename = zip_path.file_name().and_then(|n| n.to_str()).unwrap();
        log!("WARN: Removing old zip file: {}", filename);
        fs::remove_file(&zip_path).expect(&format!("Failed to delete zip file: {}", filename));
    }
    let zip_file = File::create(&zip_path).unwrap();

    let mut zip = ZipWriter::new(zip_file);

    let options = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Option::from(5));

    for (i, x) in files.iter().enumerate() {
        let relative_path = pathdiff::diff_paths(x, base_dir).unwrap();
        zip.start_file(relative_path.to_str().unwrap(), options)?;
        zip.write_all(&*fs::read(x)?)?;

        debug_log!("{}. Zipping file: {}", i + 1, x.to_str().unwrap_or("<invalid_utf8_name>"));
    }

    zip.finish().map(|_| ())
}

fn pretty_file_size(file: &Path) -> io::Result<String> {
    fs::metadata(file)
        .map(|m| {
            let size = Size::from_bytes(m.len());
            size.format().with_style(Style::Abbreviated).to_string().replace("iB", "B")
        })
}

/// Returns a number that is not greater than the number of processor cores of this machine,
/// and also not greater than the `job_amount`.
fn calculate_ideal_parallelism(job_amount: usize) -> usize {
    let available_parallelism = thread::available_parallelism();

    if let Err(error) = &available_parallelism {
        log!("WARN: Could not determine the number of CPU cores this processor has, ZipSource will fallback to single-threaded mode, which can be quite slow.\nError message: {error}");
    }

    let cores: usize = available_parallelism.ok().unwrap_or(NonZeroUsize::new(1).unwrap()).get();
    debug_log!("System Core Count: {cores}");
    MAX_CONCURRENT_THREADS.min(job_amount).max(1).min(cores)
}

#[derive(Clone, Debug)]
struct Settings {
    base_dir: PathBuf,
    zip_path: PathBuf,
}
