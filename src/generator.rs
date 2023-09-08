use std::fs;

const fn get_toml_data() -> &'static str {
r#"ip = "127.0.0.1"
port = 8080

[async]
safe_async = true

[documents]
hello = "./endpoints/hello.py"
    "#
}

const fn get_lib_data() -> &'static str {
r#"import athen_rs

def register(func=None, mode=""):
    if func:
        return athen_rs.register_func(func, "run")
    else:
        def wrapper(function):
            return athen_rs.register_func(function, mode)
        return wrapper
    "#
}

const fn get_rs_lib_data() -> &'static str {
r#"
"#
}

const fn get_demo_data() -> &'static str {
r#"from athen import register

@register(mode = "run")
def run(query: dict) -> str:
    return "Hello World!"
    "#
}

fn write_to(path: &str, contents: &str) {
    fs::write(path, contents).expect(format!("write_to(): Unable to write to file `{}`", path).as_str());
}

fn make_dir(path: &str) {
    fs::create_dir(path).expect(format!("make_dir(): Unable to make directory `{}`", path).as_str());
}

pub fn generate_files(path: &str) {
    let stripped_path = if path.ends_with("/") { path.strip_suffix("/").unwrap().to_string() } else { path.to_string() };

    // Write Athen.toml
    let toml_path = stripped_path.to_string() + "/Athen.toml";
    write_to(&toml_path.as_str(), get_toml_data());

    // Write lib pyfile
    let lib_path = stripped_path.to_string() + "/athen.py";
    let rs_lib_path = stripped_path.to_string() + "/athen_rs.py";
    make_dir("./endpoints/");
    write_to(&lib_path.as_str(), get_lib_data());
    write_to(&rs_lib_path.as_str(), get_rs_lib_data());

    // Write demo pyfile
    let py_path = stripped_path.to_string() + "/endpoints/hello.py";
    write_to(&py_path.as_str(), get_demo_data());
}