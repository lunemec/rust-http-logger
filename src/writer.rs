use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write as FWrite};
use std::path::Path;
use std::sync::mpsc::Receiver;
use std::thread;

pub fn spawn_writer(log_path: String, rx: Receiver<String>) {
    thread::spawn(move|| {
        let log_file = open_log(&log_path);
        let mut log_writer = BufWriter::new(&log_file);

        for recieved in rx.iter() {
            match log_writer.write_all(&recieved.into_bytes()) {
                Ok(_) => {},
                Err(reason) => println!("Error while writing to a log file: {}, reason: {}", log_path, reason.description())
            }
            match log_writer.flush() {
                Ok(_) => {},
                Err(reason) => println!("Error while flushing file: {}, reason: {}", log_path, reason.description())
            }
        }
    });
}

fn open_log(log_path: &String) -> File {
    let path = Path::new(log_path);
    let display = path.display();

    // Open/create LOG_FILE for writing. Panic if unable to open/create.
    let file = match OpenOptions::new()
            .read(false)
            .append(true)
            .write(true)
            .create(true)
            .open(path) {
        Err(reason) => panic!("Unable to open log file {}. Reason: {}", display,
                                                                        reason.description()),
        Ok(file) => file,
    };
    file
}