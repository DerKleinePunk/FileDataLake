use std::{ffi::CString, path::{Path, PathBuf}};
use pyo3::{Py, PyAny, PyResult, Python, types::{IntoPyDict, PyAnyMethods, PyModule}};
use std::collections::HashMap;
use std::fs;

// Todo see this make Python saver...
//https://users.rust-lang.org/t/ipc-communication-shared-memory-unix-sockets-and-separate-process/106382

pub fn run_python_file(source_file_name: &Path, analyse_file: &PathBuf, function_name: &String) -> PyResult<HashMap<String, String>> {

    log::debug!("python file: {source_file_name:?}");
    let file_text = fs::read_to_string(source_file_name)?;
    let py_app_text = CString::new(file_text).unwrap();
    let python_result = run_python_code(&py_app_text, analyse_file, function_name);

    return  python_result;
}

fn run_python_code(py_app_text: &CString, analyse_file: &PathBuf, function_name: &String) -> PyResult<HashMap<String, String>> {
    /*let sys = py.import("sys")?;
    let version: String = sys.get(py, "version")?.extract(py)?;

    let locals = PyDict::new(py);
    locals.set_item(py, "os", py.import("os")?)?;
    let user: String = py.eval("os.getenv('USER') or os.getenv('USERNAME')", None, Some(&locals))?.extract(py)?;

    log::debug!("Hello {}, I'm Python {}", user, version);*/

    let key1 = "filename";
    let val1 = analyse_file.to_str();

    Python::attach(|py| {
        /*let syspath = py
            .import("sys")?
            .getattr("path")?
            .cast_into::<PyList>()?;
        syspath.insert(0, path)?;*/
        log::debug!("Python version: {:?}", py.version());
        let fun: Py<PyAny> = PyModule::from_code(
            py,
            py_app_text.as_c_str(),
            c"example.py",
            c"",
        )?
        .getattr(function_name)?
        .into();

        // call object with PyDict
        let kwargs = [(key1, val1)].into_py_dict(py)?;
        let result :HashMap<String, String>= fun.call(py, (), Some(&kwargs))?.extract(py)?;

        /*
        let key2 = "key2";
        let val2 = 2;

        // pass arguments as Vec
        let kwargs = vec![(key1, val1), (key2, val2)];
        fun.call(py, (), Some(&kwargs.into_py_dict(py)?))?;

        // pass arguments as HashMap
        let mut kwargs = HashMap::<&str, i32>::new();
        kwargs.insert(key1, 1);
        fun.call(py, (), Some(&kwargs.into_py_dict(py)?))?;*/

        Ok(result)
    })
}
