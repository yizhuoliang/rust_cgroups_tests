use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;
use std::time::{Instant, Duration};
use std::thread;

const DIRECTORY: &str = "io_test_dir";
const FILENAME: &str = "io_test_file";
const NUMBER_OF_WORKERS: usize = 3;

fn disk_setup() -> io::Result<()> {
    fs::create_dir(DIRECTORY)?;
    for worker_id in 0..NUMBER_OF_WORKERS {
        let file_name = format!("{}_{}", FILENAME, worker_id);
        let mut file = File::create(Path::new(DIRECTORY).join(&file_name))?;
        file.write_all(b"Initial data for disk I/O testing.\n")?;
    }
    Ok(())
}


// Function to simulate disk I/O operation
fn disk_io_worker(worker_id: usize, read_enabled: Arc<AtomicBool>, write_enabled: Arc<AtomicBool>, is_running: Arc<AtomicBool>) {
    let file_name = format!("{}_{}", FILENAME, worker_id);
    let file_path = Path::new(DIRECTORY).join(file_name);
    let mut read_bytes = 0;
    let mut write_bytes = 0;
    let mut last_report = Instant::now();

    while is_running.load(Ordering::Relaxed) {
        if read_enabled.load(Ordering::Relaxed) {
            let mut file = File::open(&file_path).unwrap();
            let mut buffer = [0; 1024];
            if let Ok(bytes_read) = file.read(&mut buffer) {
                read_bytes += bytes_read;
            }
        }

        if write_enabled.load(Ordering::Relaxed) {
            let mut file = OpenOptions::new().write(true).append(true).open(&file_path).unwrap();
            if let Ok(bytes_written) = file.write(b"Some additional data...\n") {
                write_bytes += bytes_written;
            }
        }

        if last_report.elapsed() >= Duration::from_secs(5) {
            println!("Read Speed: {} bytes/sec, Write Speed: {} bytes/sec", read_bytes / 5, write_bytes / 5);
            read_bytes = 0;
            write_bytes = 0;
            last_report = Instant::now();
        }
    }
}

fn disk_cleanup() -> io::Result<()> {
    for worker_id in 0..NUMBER_OF_WORKERS {
        let file_name = format!("{}_{}", FILENAME, worker_id);
        fs::remove_file(Path::new(DIRECTORY).join(&file_name))?;
    }
    fs::remove_dir(DIRECTORY)?;
    Ok(())
}

fn main() {
    // Set up the disk
    disk_setup();

    // Create boolean flags for read and write operations, and a running flag
    let read_enabled = Arc::new(AtomicBool::new(true));
    let write_enabled = Arc::new(AtomicBool::new(true));
    let is_running = Arc::new(AtomicBool::new(true));

    // Spawn worker threads
    let mut handles = vec![];
    for worker_id in 0..NUMBER_OF_WORKERS {
        let read_clone = Arc::clone(&read_enabled);
        let write_clone = Arc::clone(&write_enabled);
        let running_clone = Arc::clone(&is_running);
        handles.push(thread::spawn(move || {
            disk_io_worker(worker_id, read_clone, write_clone, running_clone);
        }));
    }

    // wait for 'q' every second to stop the program
    println!("Press 'q' to stop the program...");
    loop {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                if input.trim() == "q" {
                    break;
                }
            },
            Err(error) => println!("Error reading from stdin: {:?}", error),
        }
        // Sleep for 1 second before the next iteration
        thread::sleep(Duration::from_secs(1));
    }

    // set running flag to false to stop the worker threads
    is_running.store(false, Ordering::Relaxed);

    // wait for the threads to finish
    for handle in handles {
        handle.join().unwrap();
    }

    // Clean up the disk before exiting
    disk_cleanup();

    println!("Program exited cleanly.");
}
