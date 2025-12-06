use std::{ffi::CString, path::{Path}};
use pyo3::{Py, PyAny, PyResult, Python, types::{IntoPyDict, PyAnyMethods, PyModule}};
use std::collections::HashMap;
use std::fs;

pub fn run_python_file(file_name: &Path) -> std::io::Result<()> {

    log::debug!("python file: {file_name:?}");
    let file_text = fs::read_to_string(file_name)?;
    let py_app_text = CString::new(file_text).unwrap();
    let python_result = run_python_code(&py_app_text);

    Ok(())
}

fn run_python_code(py_app_text: &CString) -> PyResult<()> {
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
