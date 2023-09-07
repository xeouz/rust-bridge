use pyo3::{PyErr, Python, types::{PyDict, PyTuple, PyModule}, pyclass, PyAny, Py, pymethods, PyResult, pymodule, IntoPy, wrap_pymodule};

use super::{reader::read_file, QueryData, QueryItem};

#[pyclass(name = "register_func")]
#[derive(Clone)]
pub struct PyRegisterDecorator {
    mode: String,
    wraps: Py<PyAny>
}

#[pymethods]
impl PyRegisterDecorator {
    #[new]
    pub fn __new__(wraps: Py<PyAny>, mode: Py<PyAny>) -> Self {
        let dec = PyRegisterDecorator {
            wraps: wraps, mode: mode.to_string(),
        };
        push_register_decorators(dec.clone());

        dec
    }

    pub fn get_mode(&self) -> &str { self.mode.as_str() }
    pub fn get_wraps(&self) -> &Py<PyAny> { &self.wraps }

    #[pyo3(signature = (*_args, **_kwargs))]
    fn __call__(
        &self,
        py: Python<'_>,
        _args: &PyTuple,
        _kwargs: Option<&PyDict>,
    ) -> PyResult<Py<PyAny>> {
        let ret = py.eval("0", None, None).unwrap().into_py(py);
        Ok(ret)
    }
}

#[pymodule]
pub fn athen_rs_module(_py: Python<'_>, module: &PyModule) -> PyResult<()> {
    module.add_class::<PyRegisterDecorator>()?;
    Ok(())
}

#[derive(Debug)]
pub struct PyExecutionError {
    pub message: String
}

impl From<PyErr> for PyExecutionError {
    fn from(value: PyErr) -> Self {
        PyExecutionError { message: value.to_string() }
    }
}

#[derive(Debug)]
pub struct PushRegisterDecoratorsError;
static mut GLOBAL_DECORATORS: Vec<PyRegisterDecorator> = vec![];

impl IntoPy<Py<PyAny>> for QueryItem {
    fn into_py(self, py: Python<'_>) -> Py<PyAny> {
        match self {
            super::QueryItem::Number(num) => num.as_f64().into_py(py),
            super::QueryItem::String(string) => string.into_py(py),
        }
    }
}

pub fn initiate_python() -> Result<Py<PyAny>, PyErr> {
    pyo3::append_to_inittab!(athen_rs_module);
    pyo3::prepare_freethreaded_python();

    let ret = Python::with_gil(|py| -> Result<_, PyErr> {
        let athen_rs = wrap_pymodule!(athen_rs_module)(py).into_ref(py);
        let sys = PyModule::import(py, "sys")?;
        let py_modules: &PyDict = sys.getattr("modules")?.downcast()?;
        py_modules.set_item("athen_rs", athen_rs)?;

        let result = read_file("./athen.py");
        if result.is_err() { panic!() }
        let lib_code = result.unwrap();
        let lib = PyModule::from_code(py, &lib_code, "athen.py", "athen")?;
        py_modules.set_item("athen", lib)?;

        let asyncio = PyModule::import(py, "asyncio")?;
        let event_loop = asyncio.call_method0("new_event_loop")?.into_py(py);

        Ok(event_loop)
    })?;

    Ok(ret)
}

pub fn run_python(code: &str) -> Result<(), PyExecutionError> {
    Python::with_gil(|py| -> Result<(), PyExecutionError> {
        let res = Python::run(py, code, None, None);

        if res.is_err() {
            return Err(PyExecutionError { message: res.unwrap_err().value(py).to_string() });
        }

        Ok(())
    })?;

    Ok(())
}

pub async fn call_function(function: &Py<PyAny>, query_arg: QueryData, is_init: bool, event_loop: Option<&Py<PyAny>>) -> Result<Py<PyAny>, PyExecutionError> {
    let ret = Python::with_gil(|py| -> PyResult<_> {
        let args = PyTuple::new(py, vec![query_arg.inner.into_py(py)]);
        
        let result = if !is_init {
            function.call1(py, args)
        }
        else {
            function.call0(py)
        }?;

        let inspect = py.import("inspect")?;
        let is_coro = inspect.call_method1("iscoroutine", (&result,))?;
        let ret = if !is_coro.is_true()? {
            result
        }
        else {
            let locals = PyDict::new(py); locals.set_item("coro", result)?; locals.set_item("loop", event_loop.unwrap())?;
            py.eval("loop.run_until_complete(coro)", None, Some(locals))?.into_py(py)
        };

        Ok(ret)
    })?;

    Ok(ret)
}

pub fn get_register_decorators() -> &'static Vec<PyRegisterDecorator> {
    unsafe { &GLOBAL_DECORATORS }
}

pub fn push_register_decorators(value: PyRegisterDecorator) {
    unsafe {
        GLOBAL_DECORATORS.push(value);
    }
}