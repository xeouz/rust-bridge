use pyo3::{Py, PyAny};
use rocket::{State, Route, http::{Method, uri::Query}, route::Handler, route::Outcome, Request, Data};
use serde_json::Number;
use std::collections::HashMap;
use serde::Deserialize;

use self::{pyinterface::{run_python, initiate_python, get_register_decorators, call_function}, reader::{FileReadError, read_toml, HydratedConfig, read_file}};

pub mod pyinterface;
pub mod reader;
pub mod net;

///-- Data --///
pub struct GlobalCollection {
    documents: HashMap<String, DataDocument>,
    config: HydratedConfig,
}
pub struct DataDocument {
    name: String,
    function: Py<PyAny>,
}
impl GlobalCollection {
    pub fn new(config: HydratedConfig) -> Self {
        GlobalCollection { documents: HashMap::new(), config: config }
    }

    pub fn get_document(&self, name: &str) -> &DataDocument { 
        &self.documents.get(name).expect(format!("GlobalCollection: get_document(): Could not get document by name {}", name).as_str())
    }
    pub fn get_config(&self) -> &HydratedConfig { &self.config }

    pub fn insert_document(&mut self, doc: DataDocument) {
        self.documents.insert(doc.name.to_string(), doc);
    }
}
impl DataDocument {
    pub fn new(name: String, function: Py<PyAny>) -> Self {
        DataDocument { name: name, function: function }
    }

    pub async fn execute_empty(&self) {
        let _ = call_function(&self.function, QueryData::default(), true).await;
    }
    pub async fn execute(&self, query: QueryData, is_init: bool) -> String {
        call_function(&self.function, query, is_init).await.unwrap().to_string()
    }
}

///-- GET Handlers --///
#[derive(Clone)]
struct QueryHandler {
    document_name: String,
}

#[rocket::async_trait]
impl Handler for QueryHandler {
    async fn handle<'r>(&self, req: &'r Request<'_>, _data: Data<'r>) -> Outcome<'r> {
        let col = req.guard::<&State<GlobalCollection>>().await
                                                           .map(|collection| collection).unwrap().inner();

        Outcome::from(req, execute_query(col, &self.document_name, extract_query(req.uri().query().unwrap())).await)
    }
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum QueryItem {
    Number(Number),
    String(String)
}

#[derive(Deserialize, Debug)]
pub struct QueryData {
    #[serde(flatten)]
    inner: HashMap<String, QueryItem>,
}

impl Default for QueryData {
    fn default() -> Self {
        QueryData { inner: HashMap::new() }
    }
}

#[get("/")]
pub async fn index(_collection: &State<GlobalCollection>) -> String {
    tokio::time::sleep(std::time::Duration::new(5, 0)).await;
    "Hello World".to_string()
}

///-- Function Handlers --///
pub async fn initiate() -> Result<(GlobalCollection, Vec<Route>), FileReadError> {
    let _ = initiate_python().expect("Python Initialization Error Occurred");

    let config = read_toml("./")?;
    let _ = run_programs(&config)?;

    let mut routes: Vec<Route> = routes![index];
    let mut collection = GlobalCollection::new(config.clone());
    let decs = get_register_decorators();
    let mut index = 0;
    for dec in decs.iter() {
        let name = config.get_documents()[index].0.to_string();
        let doc = DataDocument::new(name.to_string(), dec.get_wraps().clone());

        if dec.get_mode() == "init" {
            doc.execute_empty().await;
            continue;
        }
        
        collection.insert_document(doc);
        routes.push(create_route(&name));

        index += 1;
    };

    Ok((collection, routes))
}

pub fn create_route(document: &str) -> Route {
    let route_path = "/".to_string() + document + "/<query>";
    let handler = QueryHandler { document_name: document.to_string() };
    let route = Route::new(Method::Get, &route_path, handler);

    route
}

fn run_programs(config: &HydratedConfig) -> Result<(), FileReadError> {
    let docs = config.get_documents();
    for (_name, pyfile) in docs {
        let code = read_file(&pyfile)?;
        let err = run_python(code.as_str());

        if err.is_err() {
            println!("Python Error: {}", err.unwrap_err().message);
            panic!()
        }
    };

    Ok(())
}

fn extract_query(query: Query) -> QueryData {
    let query_data: Vec<_> = query.segments().collect();
    let q = query_data.get(0).unwrap_or(&("q", "")).1;
    let json: QueryData = serde_json::from_str(q).unwrap();

    json
}

async fn execute_query(collection: &GlobalCollection, document_name: &str, query: QueryData) -> String {
    let doc = collection.get_document(document_name);
    doc.execute(query, false).await
}