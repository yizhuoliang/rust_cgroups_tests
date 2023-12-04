use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write, Seek};
use std::path::Path;
use std::time::{Instant, Duration};
use std::thread;

const DIRECTORY: &str = "io_test_dir";
const FILENAME: &str = "io_test_file";
const NUMBER_OF_WORKERS: usize = 3;

fn disk_setup() -> io::Result<()> {
    fs::create_dir(DIRECTORY)?;
    let data_chunk = vec![0u8; 8 * 1024 * 1024]; // 8 MB chunk
    for worker_id in 0..NUMBER_OF_WORKERS {
        let file_name = format!("{}_{}", FILENAME, worker_id);
        let file_path = Path::new(DIRECTORY).join(&file_name);
        let mut file = File::create(&file_path)?;

        for _ in 0..16 { // Write 16 times 8 MB to make 128 MB
            file.write_all(&data_chunk)?;
        }
    }
    Ok(())
}


// Function to simulate disk I/O operation
fn disk_io_worker(worker_id: usize, read_enabled: Arc<AtomicBool>, write_enabled: Arc<AtomicBool>, is_running: Arc<AtomicBool>) {
    let file_name = format!("{}_{}", FILENAME, worker_id);
    let file_path = Path::new(DIRECTORY).join(&file_name);
    let mut read_bytes = 0;
    let mut write_bytes = 0;
    let mut last_report = Instant::now();
    let mut read_position = 0;

    while is_running.load(Ordering::Relaxed) {
        if read_enabled.load(Ordering::Relaxed) {
            let mut file = File::open(&file_path).unwrap();
            file.seek(io::SeekFrom::Start(read_position)).unwrap();
            let mut buffer = vec![0; 8 * 1024 * 1024]; // 8 MB buffer
            match file.read(&mut buffer) {
                Ok(bytes_read) => {
                    read_bytes += bytes_read;
                    read_position += bytes_read as u64;
                    if read_position >= 128 * 1024 * 1024 { // Reset after 128 MB
                        read_position = 0;
                    }
                },
                Err(_) => break,
            }
        }

        if write_enabled.load(Ordering::Relaxed) {
            let mut file = OpenOptions::new().write(true).append(true).open(&file_path).unwrap();
            if let Ok(bytes_written) = file.write(b"Some additional data...\n") {
                write_bytes += bytes_written;
            }
        }

        if last_report.elapsed() >= Duration::from_secs(5) {
            let read_speed_mbps = (read_bytes as f64) / 5.0 / 1_048_576.0;
            let write_speed_mbps = (write_bytes as f64) / 5.0 / 1_048_576.0;
            println!("Read Speed: {:.2} MB/s, Write Speed: {:.2} MB/s", read_speed_mbps, write_speed_mbps);
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

fn disk_experiment() {
     // Set up the disk
     disk_setup();

     // Create boolean flags for read and write operations, and a running flag
     let read_enabled = Arc::new(AtomicBool::new(true));
     let write_enabled = Arc::new(AtomicBool::new(false));
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
     println!("Press 'q' and return to cleanup and terminate");
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

fn cpu_worker(worker_id: usize, is_running: Arc<AtomicBool>) {
    let mut rounds = 0u64;
    let mut last_report = Instant::now();

    while is_running.load(Ordering::Relaxed) {
        // Perform a CPU-intensive task
        cpu_intensive_task();
        rounds += 1;

        // Check if it's time to report
        if last_report.elapsed() >= Duration::from_secs(5) {
            println!("Worker {}: {} rounds of computation in last 5 seconds", worker_id, rounds);
            rounds = 0;
            last_report = Instant::now();
        }
    }
}

fn cpu_intensive_task() -> u64 {
    // Example of a CPU-intensive task (this is just a placeholder)
    let mut sum = 0;
    for i in 0..100_000 {
        sum += i;
    }
    sum // Count this as one round
}

fn cpu_experiment() {
    let is_running = Arc::new(AtomicBool::new(true));

    // Start the worker
    let is_running_clone = Arc::clone(&is_running);
    let worker_thread = thread::spawn(move || {
        cpu_worker(1, is_running_clone);
    });

    // Let the worker run for some time...
    thread::sleep(Duration::from_secs(20));

    // Stop the worker
    is_running.store(false, Ordering::Relaxed);
    worker_thread.join().unwrap();
}

fn main() {
   cpu_experiment();
}
