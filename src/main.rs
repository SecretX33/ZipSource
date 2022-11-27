mod chars;

use std::{env, fs, io, thread};
use std::collections::HashSet;
use std::fs::{File};
use std::io::Write;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, mpsc};
use std::sync::atomic::{AtomicU32, Ordering};
use log::{debug, info, warn};
use pretty_env_logger::env_logger::Target;
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
    pretty_env_logger::init();

    info!("Started collecting files");

    let base_dir = get_base_dir();

    let all_files = list_files_recursive(&base_dir);
    let all_files_path = all_files.iter().map(|p| p.as_path()).collect();
    info!("Found a total of {} files in the folder.", all_files.len());

    let valid_files = filter_valid_files(&base_dir, all_files_path);
    info!("There are {} files to be zipped (out of {}, which means {} files were ignored)", valid_files.len(), all_files.len(), all_files.len() - valid_files.len());

    let folder_name = base_dir.file_name().and_then(|n| n.to_str())
        .map(|n| n.to_string())
        .unwrap();
    let zip_path = base_dir.clone().join(folder_name + " (Source Code).zip");

    zip_files(&zip_path, &base_dir, valid_files).expect("Creation of zip failed");

    let file_size = pretty_file_size(&zip_path).expect("Could not read zip file info");
    println!("Zip was successfully created (file size is {})!", file_size.replace("iB", "B"))
}

/// Get the base directory that should be used in this program based on the first console argument
/// (which should be a path), and if not provided then it defaults to the current directory.
fn get_base_dir() -> PathBuf {
    let args: Vec<_> = env::args().into_iter().skip(1).collect();

    // If user provided the base path, let's use it
    if let Some(arg_path) = args.first() {
        return PathBuf::from(arg_path);
    }

    // User didn't provide the base path, default to current directory
    env::current_dir().expect("Current dir is not set")
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
    debug!("There are {} chunks of files to be processed (thread pool size: {})", chunks_amount, pool_size);

    let pool = ThreadPool::new(pool_size);
    let count = Arc::from(AtomicU32::new(0));
    let (tx, rx) = mpsc::channel();

    for files in files_chunked {
        let tx = tx.clone();
        let count = Arc::clone(&count);

        pool.execute(move|| {
            let test: HashSet<&Path> = files.iter().map(|p| p.as_path()).collect();
            let mut command = build_check_ignore_command(&test);
            let ignored_files = run_ignored_files_command(&mut command);
            debug!("[{}] Chunk size: {}, ignored files size: {}", count.fetch_add(1, Ordering::Relaxed), files.len(), ignored_files.len());

            tx.send(ignored_files).expect("channel will be there waiting for the pool");
        });
    }

    // Await for all jobs to finish
    pool.join();

    // Destroy the thread pool eagerly
    drop(pool);

    // Collect their results
    let ignored_files: HashSet<_> = rx.iter().take(chunks_amount).flatten().collect();
    let ignored_files: HashSet<&Path> = ignored_files.iter().map(|e| e.as_path()).collect();

    debug!("\nall_files: {:?}", all_files.iter().take(8).collect::<Vec<_>>());
    debug!("\nignored_files: {:?}\n\n", ignored_files.iter().take(8).collect::<Vec<_>>());

    let valid_files: HashSet<&Path> = all_files_vector.iter()
        .copied()
        .filter(|&path| !ignored_files.contains(path) && !is_excluded_specially(current_dir, path))
        .collect();

    debug!(
        "ignored_files: {}, all_files_vector size: {}, valid_files size: {}",
        ignored_files.len(),
        all_files_vector.len(),
        valid_files.len()
    );
    valid_files
}

fn build_check_ignore_command(files: &HashSet<&Path>) -> Command {
    let path_list: Vec<&str> = files.iter()
        .filter_map(|path| path.to_str())
        .collect();

    let (cmd, first_arg) = if cfg!(target_os = "windows") { ("cmd", "/C") } else { ("sh", "-c") };
    let args = [[first_arg, "git", "check-ignore"].as_slice(), path_list.as_slice()].concat();

    let mut command = Command::new(cmd);
    command.args(args);
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
    let paths_to_exclude = [current_dir.to_path_buf().join(".git")];

    paths_to_exclude.iter().any(|p| file_to_check.starts_with(p))
}

fn zip_files(filename: &Path, base_dir: &Path, files: HashSet<&Path>) -> ZipResult<()> {
    if filename.exists() {
        warn!("Removing old zip file: {}", filename.file_name().unwrap().to_str().unwrap());
        fs::remove_file(filename).expect(&format!("Failed to delete zip file: {}", filename.to_str().unwrap()));
    }
    let zip_file = File::create(filename).unwrap();

    let mut zip = ZipWriter::new(zip_file);

    let options = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Option::from(5));

    println!();

    for (i, x) in files.iter().enumerate() {
        let relative_path = pathdiff::diff_paths(x, base_dir).unwrap();
        zip.start_file(relative_path.to_str().unwrap(), options)?;
        zip.write_all(&*fs::read(x)?)?;

        debug!("{}. Zipping file: {}", i + 1, x.to_str().unwrap_or("<invalid_utf8_name>"));
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
    let cores = thread::available_parallelism().ok().unwrap_or(NonZeroUsize::new(1).unwrap()).get();
    debug!("System Core Count: {cores}");
    MAX_CONCURRENT_THREADS.min(job_amount).max(1).min(cores)
}
