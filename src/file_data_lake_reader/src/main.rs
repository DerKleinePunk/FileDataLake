use notify::event::CreateKind;
use notify::{Config, EventKind, RecursiveMode, Watcher};
use pyo3::prelude::*;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::runtime::{self, Builder};
use crate::app_dtos::FileEntry;
use crate::database_handler::LocalDbState;
use crate::helper::string_ify_ioerror;

//Hints
//https://docs.rs/workerpool/latest/workerpool/
//https://pyo3.rs/main/python-from-rust/calling-existing-code.html
//https://github.com/raddevus/watcher/blob/main/src/main.rs

mod app_dtos;
mod database_handler;
mod helper;
mod new_file_worker;
mod python_runner;
mod image_handler;

/// `AppConfigFile` implements `Default`
impl ::std::default::Default for AppConfigFile {
    fn default() -> Self {
        Self {
            version: 0,
            database: "fdl.db3".into(),
            watch_path: ".".to_string(),
            python_path: None,
            thumbnail_path: None
        }
    }
}

#[derive(Serialize, Deserialize)]
struct AppConfigFile {
    version: u64,
    database: String,
    watch_path: String,
    python_path: Option<String>,
    thumbnail_path: Option<String>,
}

struct SharedData {
    python_path: String,
    thumbnail_path: String
}

impl SharedData {
    pub fn new(python_path: &String, thumbnail_path: &String) -> SharedData {
        SharedData {
            python_path: python_path.clone(),
            thumbnail_path: thumbnail_path.clone()
        }
    }
}

struct AccessSharedData {
    pub sd: Arc<Mutex<SharedData>>,
}

impl AccessSharedData {
    fn clone(&self) -> Self {
        AccessSharedData {
            sd: Arc::clone(&self.sd),
        }
    }

    pub fn python_path(&self) -> String {
        let lock = self.sd.lock().unwrap();
        lock.python_path.clone()
    }

    pub fn thumbnail_path(&self) -> String {
        let lock = self.sd.lock().unwrap();
        lock.thumbnail_path.clone()
    }
}

#[tokio::main]
async fn main() -> Result<(),()> {
    //Todo Put to Config
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    let n_workers = 4;

    println!("File Reader starting");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let config_path = confy::get_configuration_file_path("fdl", "reader");
    match config_path {
        Ok(result) => log::debug!("Config is hier {result:?}"),
        Err(error) => {
            eprintln!("Error: {error:?}");
            return Err(());
        }
    }

    let mut cfg: AppConfigFile = confy::load("fdl", "reader").unwrap();

    let file_watch_path: &str;
    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        file_watch_path = &args[1];

        let target_dir_exists = Path::new(file_watch_path).exists();
        if !target_dir_exists {
            log::error!("Error: dir not exits");
            eprintln!("Error: {}", "dir not exits");
            return Err(());
        }
        cfg.watch_path = file_watch_path.to_string();
    } else {
        file_watch_path = cfg.watch_path.as_str();
    }

    let python_path = "".to_string();
    let thumbnail_path = "".to_string();

    if cfg.python_path == None {
        cfg.python_path = Some(python_path.clone());
    }

    if cfg.thumbnail_path == None {
        cfg.thumbnail_path = Some(thumbnail_path.clone());
    }

    //Save so User see the new Defaults
    confy::store("fdl", "reader", &cfg).unwrap();

    let thread_pool: runtime::Runtime = Builder::new_multi_thread()
    .worker_threads(n_workers)
    .thread_keep_alive(Duration::from_secs(30))
    .build().unwrap();

    //Starting Python
    Python::initialize();

    log::debug!("Starting Database");

    let dbfile: &Path = cfg.database.as_ref();
    let mut dbstate = database_handler::LocalDbState::new(&dbfile);
    let result = database_handler::LocalDbState::create_database(&mut dbstate).await;
    if result.is_err() {
        let error = result.err();
        log::error!("Error: {error:?}");
        eprintln!("Error: {error:?}");
        return Err(());
    }
    //Todo check DB Version and try Update



    let common_data = SharedData::new(&python_path, &thumbnail_path);

    let shared_data = AccessSharedData {
        sd: Arc::new(Mutex::new(common_data)),
    };

    log::debug!("we watch at {file_watch_path:?}");

    println!("Watching {file_watch_path:?} and pool is running");
    println!("Waiting for Ctrl-C...");

    if let Err(error) = watch(file_watch_path, running, thread_pool, shared_data, dbstate) {
        log::error!("Error: {error:?}");
    }

   println!("Got it! Exiting...");

   Ok(())
}

fn watch<P: AsRef<Path>>(
    path: P,
    test: Arc<AtomicBool>,
    worker_pool: runtime::Runtime,
    shared_data: AccessSharedData,
    dbstate: database_handler::LocalDbState,
) -> notify::Result<()> {
    let (tx, receiver) = std::sync::mpsc::channel();

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher =
        notify::RecommendedWatcher::new(tx, Config::default().with_compare_contents(true))?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    let dbpool = dbstate.get();
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
                            let shared_data_clone = shared_data.clone();
                            let pool_manager_clone = dbpool.clone();
                            worker_pool.spawn(new_file_event(
                                        event_ok.paths[0].clone(),
                                        shared_data_clone.clone(),
                                        pool_manager_clone,
                                    ));
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

async fn new_file_event(
    path: PathBuf,
    shared_data: AccessSharedData,
    pool: deadpool_sqlite::Pool,
) {
    // I hope the display all errors...
    let result = new_file_hander(path, shared_data, pool).await;
    if result.is_err() {
        log::error!("{:?}", result.err())
    }
}

//https://medium.com/better-programming/easy-multi-threaded-shared-memory-in-rust-57344e9e8b97
//<P: AsRef<Path>>
async fn new_file_hander(
    path: PathBuf,
    shared_data: AccessSharedData,
    pool: deadpool_sqlite::Pool,
) -> Result<(),String> {
    let python_path = shared_data.python_path();
    log::debug!("using python path {python_path:?}");

    //Todo move to new file worker or To lib for cli using

    let mut file_attributes: HashMap<String,String> = HashMap::new();

    let file_size = new_file_worker::print_file_size(&path).map_err(string_ify_ioerror)?;
    if helper::is_file_image(&path) {
        file_attributes.insert("Image".to_string(), "true".to_string());

        let mut path2 = path.parent().unwrap().to_path_buf();
        let mut test_file_name = path.file_prefix().unwrap().to_os_string();
        test_file_name.push("_tbn.jpg");
        path2.push(test_file_name);

        let image_size  = image_handler::make_thumbnail(&path, &path2)?;

        file_attributes.insert("Width".to_string(), image_size.width.to_string());
        file_attributes.insert("Heigth".to_string(), image_size.heigth.to_string());
    }

    let app_exe = env::current_exe().map_err(string_ify_ioerror)?;
    let app_path = app_exe.parent().unwrap();
    let pysourcepath = app_path.join("../../python/");
    let file_name = pysourcepath.join("example.py");
    let python_result = python_runner::run_python_file(&file_name, &path);
    match python_result {
        Ok(_) => {
            log::debug!("Python with no error");
        }
        Err(error) => {
            log::error!("Python with error {error:?}");
            return Err("Python excute Error".to_string());
        }
    }

    let python_attributes = python_result.unwrap();

    let mut file_entry = FileEntry::new();
    file_entry.name = path.file_name().unwrap().to_str().unwrap().to_string();
    file_entry.size = file_size;
    file_entry.hash = helper::sha256_digest(&path).unwrap();
    for p_attrib in &python_attributes {
        file_entry
            .attributes
            .insert(p_attrib.0.to_string(), p_attrib.1.to_string());
    }

    for p_attrib in &file_attributes {
        file_entry
            .attributes
            .insert(p_attrib.0.to_string(), p_attrib.1.to_string());
    }

    log::debug!("File Entry bevor save {file_entry:?}");

    let result_insert = LocalDbState::save_file_info(pool, file_entry).await;
    match result_insert {
        Ok(_) => {
            log::debug!("database with no error");
        }
        Err(error) => {
            log::error!("database with error {error:?}");
            return Err("database with error".to_string());
        }
    }

    Ok(())
}
