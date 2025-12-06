use notify::event::CreateKind;
use notify::{Config, EventKind, RecursiveMode, Watcher};
use pyo3::prelude::*;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use workerpool::Pool;
use workerpool::thunk::{Thunk, ThunkWorker};
use confy::ConfyError;
use serde_derive::{Serialize, Deserialize};

use crate::app_dtos::FileEntry;

//Hints
//https://docs.rs/workerpool/latest/workerpool/
//https://pyo3.rs/main/python-from-rust/calling-existing-code.html
//https://github.com/raddevus/watcher/blob/main/src/main.rs

mod new_file_worker;
mod python_runner;
mod database_handler;
mod app_dtos;

/// `AppConfigFile` implements `Default`
impl ::std::default::Default for AppConfigFile {
    fn default() -> Self { Self { version: 0, database: "fdl.toml".into() } }
}

#[derive(Serialize, Deserialize)]
struct AppConfigFile {
    version: u64,
    database: String,
}


fn main() -> std::io::Result<()> {
    //Todo Put to Config
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    let n_workers = 2;

    println!("File Reader starting");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let mut file_watch_path = ".";

    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        file_watch_path = &args[1];

        let target_dir_exists = Path::new(file_watch_path).exists();
        if !target_dir_exists {
            eprintln!("Error: {}", "dir not exits");
            //Todo make it better
            std::process::exit(exitcode::DATAERR);
            //return Err(());
        }
    }

    let cfg: AppConfigFile = confy::load("fdl", "reader").unwrap();

    let worker_pool = Pool::<ThunkWorker<()>>::new(n_workers);

    //Starting Python
    Python::initialize();

    println!(
        "Watching {} and pool is running",
        Path::new(file_watch_path).display()
    );

    log::debug!("Starting Database");

    let db_file_name = Path::new(&cfg.database);
    let mut dbstate = database_handler::LocalDbState::new(&db_file_name);
    let result = database_handler::LocalDbState::create_database(&mut dbstate);
    if result.is_err() {
        let error = result.err();
        log::error!("Error: {error:?}");
        //Todo make it better
        std::process::exit(exitcode::DATAERR);
    }
    //Todo check DB Version and try Update

    println!("Waiting for Ctrl-C...");

    if let Err(error) = watch(file_watch_path, running, worker_pool) {
        log::error!("Error: {error:?}");
    }

    println!("Got it! Exiting...");

    Ok(())
}

//<P: AsRef<Path>>
fn new_file_hander(path: &Path) -> notify::Result<()> {
    let file_size = new_file_worker::print_file_size(path)?;

    let app_exe = env::current_exe()?;
    let app_path = app_exe.parent().unwrap();
    let pysourcepath = app_path.join("../../python/");
    let file_name = pysourcepath.join("example.py");
    let python_result = python_runner::run_python_file(&file_name);

    let mut file_entry = FileEntry::new();
    file_entry.name = path.file_name().unwrap().to_str().unwrap().to_string();
    file_entry.size = file_size;

    println!("test {file_entry:?}");

    Ok(())
}


fn watch<P: AsRef<Path>>(
    path: P,
    test: Arc<AtomicBool>,
    worker_pool: Pool<ThunkWorker<()>>,
) -> notify::Result<()> {
    let (tx, receiver) = std::sync::mpsc::channel();

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher =
        notify::RecommendedWatcher::new(tx, Config::default().with_compare_contents(true))?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    while test.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(500));
        let mut iter = receiver.try_iter();
        if iter.next().is_none() {
            continue;
        }
        while let Some(event) = iter.next() {
            match event {
                Ok(event_ok) => {
                    log::info!("New Event: {event_ok:?}");
                    //event_ok.need_rescan();
                    //todo start Work
                    match event_ok.kind {
                        EventKind::Create(CreateKind::Any) => {
                            worker_pool.execute(Thunk::of(move || {
                                let _ = new_file_hander(&event_ok.paths[0]);
                            }));
                        }
                        _other => {
                            log::debug!("Event not handeled");
                        }
                    }
                }
                Err(error) => {
                    log::error!("Error: {error:?}");
                    break;
                }
            }
        }
    }

    //Blocking All
    /*for res in receiver {
        match res {
            Ok(event) => log::info!("Change: {event:?}"),
            Err(error) => log::error!("Error: {error:?}"),
        }
    }*/

    Ok(())
}
