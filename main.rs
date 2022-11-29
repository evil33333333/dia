extern crate stopwatch;

use std::thread;
use std::io::Write;
use stopwatch::Stopwatch;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::{fs::{self, ReadDir}, path::PathBuf};

const BYTES_IN_GB: u64 = 1073741824;
#[allow(non_upper_case_globals)]
static dirs_scanned: Mutex<u64> = Mutex::new(0);
#[allow(non_upper_case_globals)]
static files_scanned: Mutex<u64> = Mutex::new(0);

fn main() {
    let mut sw: Stopwatch = Stopwatch::start_new();
    let mut dirmap: BTreeMap<u64, String> = BTreeMap::new();
    let paths: ReadDir = fs::read_dir("./").unwrap();
    let finished_arc: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));

    let finished_clone: Arc<Mutex<bool>> = Arc::clone(&finished_arc);
    thread::spawn(move || {
        loop {
            let finished: MutexGuard<bool> = finished_clone.lock().unwrap();
            if !*finished {
                let dirs: MutexGuard<u64> = dirs_scanned.lock().unwrap();
                let files: MutexGuard<u64> = files_scanned.lock().unwrap();
                print!("[dia] [dirs scanned: {} || files scanned: {} || time elapsed: {:?}] \r", *dirs, *files, sw.elapsed());
                std::io::stdout().flush().unwrap();
            }
            else {
                println!("");
                break
            }
            thread::sleep(std::time::Duration::from_millis(5));
        }
    });
    

    for path in paths {

        let path_obj: PathBuf = path.unwrap().path();
        if path_obj.is_dir() {

            let arc_size: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
            find_directory_size(&path_obj, &arc_size);

            let path_str: String = String::from(path_obj.to_str().unwrap());
            let size: MutexGuard<u64> = arc_size.lock().unwrap();
            dirmap.insert(*size as u64, path_str);
        }
    }

    sw.stop();
    {
        let mut finished: MutexGuard<bool> = finished_arc.lock().unwrap();
        *finished = true;
    }
    thread::sleep(std::time::Duration::from_secs(1));


    println!("[!] Finished all allocated work... [Duration: {:?}]", sw.elapsed());
    for (size_in_bytes, dir_name) in dirmap.iter().rev() {
        let gbs_used: f64 = *size_in_bytes as f64 / BYTES_IN_GB as f64;
        println!("[!] Directory: [{}] uses up {} GBs of space", dir_name, gbs_used);
    }
}

fn find_directory_size(path: &PathBuf, size: &Arc<Mutex<u64>>) {
    let dir = path.to_str().unwrap();
    let mut handles: Vec<thread::JoinHandle<()>> = Vec::new();

    let paths_result: Result<ReadDir, std::io::Error> = fs::read_dir(dir);

    if paths_result.is_ok() {
        let paths: ReadDir = paths_result.unwrap();
        for p in paths {
            let path_obj: PathBuf = p.unwrap().path();
            if path_obj.is_file() {
                let mut files: MutexGuard<u64> = files_scanned.lock().unwrap();
                *files = *files + 1;


                let size: Arc<Mutex<u64>> = Arc::clone(&size);
                let mut num: MutexGuard<u64> = size.lock().unwrap();
                let file_size: u64 = fs::metadata(path_obj).unwrap().len();
                *num = *num + file_size;
                
                
            }
            else if path_obj.is_dir() {
                let mut num: MutexGuard<u64> = dirs_scanned.lock().unwrap();
                *num = *num + 1;
                
                
                let size: Arc<Mutex<u64>> = Arc::clone(&size);
                let handle: thread::JoinHandle<()> = thread::spawn(move || {
                    find_directory_size(&path_obj, &size);
                });
                handles.push(handle);
                
            }
        }
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
