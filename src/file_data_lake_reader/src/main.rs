use notify::event::CreateKind;
use notify::{Config, EventKind, RecursiveMode, Watcher};
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use std::collections::HashMap;
use std::env;
use std::ffi::CString;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use std::fs;
use workerpool::Pool;
use workerpool::thunk::{Thunk, ThunkWorker};
use pyo3::types::PyList;

//Hints
//https://docs.rs/workerpool/latest/workerpool/
//https://pyo3.rs/main/python-from-rust/calling-existing-code.html

pub mod new_file_worker;

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

    let worker_pool = Pool::<ThunkWorker<()>>::new(n_workers);

    //Starting Python
    Python::initialize();

    println!(
        "Watching {} and pool is running",
        Path::new(file_watch_path).display()
    );

    println!("Waiting for Ctrl-C...");

    if let Err(error) = watch(file_watch_path, running, worker_pool) {
        log::error!("Error: {error:?}");
    }

    println!("Got it! Exiting...");

    Ok(())
}

fn new_file_hander<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    let fileSize = new_file_worker::print_file_size(path);

    let app_exe = env::current_exe()?;
    let app_path = app_exe.parent().unwrap();
    let pysourcepath = app_path.join("../../python/");
    let file_name = pysourcepath.join("example.py");
    log::debug!("python file: {file_name:?}");
    let file_text = fs::read_to_string(file_name)?;
    let py_app_text = CString::new(file_text).unwrap();
    let python_result = hello_from_python(&py_app_text);

    Ok(())
}

fn hello_from_python(py_app_text: &CString) -> PyResult<()> {
    /*let sys = py.import("sys")?;
    let version: String = sys.get(py, "version")?.extract(py)?;

    let locals = PyDict::new(py);
    locals.set_item(py, "os", py.import("os")?)?;
    let user: String = py.eval("os.getenv('USER') or os.getenv('USERNAME')", None, Some(&locals))?.extract(py)?;

    log::debug!("Hello {}, I'm Python {}", user, version);*/

    let key1 = "key1";
    let val1 = 1;
    let key2 = "key2";
    let val2 = 2;

    Python::attach(|py| {
        /*let syspath = py
            .import("sys")?
            .getattr("path")?
            .cast_into::<PyList>()?;
        syspath.insert(0, path)?;*/
        let fun: Py<PyAny> = PyModule::from_code(
            py,
            py_app_text.as_c_str(),
            c"example.py",
            c"",
        )?
        .getattr("example")?
        .into();

        // call object with PyDict
        let kwargs = [(key1, val1)].into_py_dict(py)?;
        fun.call(py, (), Some(&kwargs))?;

        // pass arguments as Vec
        let kwargs = vec![(key1, val1), (key2, val2)];
        fun.call(py, (), Some(&kwargs.into_py_dict(py)?))?;

        // pass arguments as HashMap
        let mut kwargs = HashMap::<&str, i32>::new();
        kwargs.insert(key1, 1);
        fun.call(py, (), Some(&kwargs.into_py_dict(py)?))?;

        Ok(())
    })
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
                    //todo start Work
                    match event_ok.kind {
                        EventKind::Create(CreateKind::Any) => {
                            worker_pool.execute(Thunk::of(move || {
                                let _ = new_file_hander(event_ok.paths[0].clone());
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
