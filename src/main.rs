#[macro_use]
extern crate rocket;
extern crate dotenv;

use bson::oid::ObjectId;
use mongodb::results::{DeleteResult, InsertOneResult};
use mongodb::Collection;
use rocket::fs::{relative, FileServer, NamedFile};
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::State;

use bson::doc;
use dotenv::dotenv;
use mongodb::{
    options::{ClientOptions, ResolverConfig},
    Client,
};

use std::env;
use std::error::Error;
use std::path::Path;
use std::process::Command;
use std::str;
use std::sync::Mutex;
struct PhotoKey<'r>(&'r str);

#[derive(Deserialize)]
struct Message {
    body: Mutex<String>,
}

#[derive(Deserialize)]
struct DeleteFilter {
    name: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct User {
    _id: ObjectId,
    name: String,
    contacts: Vec<String>,
    password: String,
}

#[derive(Debug)]
struct UsersCol {
    col: Collection<User>,
}

#[derive(Deserialize)]
struct Factors {
    dividend: f32,
    divisor: f32,
}

#[derive(Debug)]
enum PhotoKeyError {
    Negative,
    Missing,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for PhotoKey<'r> {
    type Error = PhotoKeyError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        fn is_valid(key: &str) -> bool {
            key == "yes"
        }

        match req.headers().get_one("nice-photo") {
            None => Outcome::Error((Status::BadRequest, PhotoKeyError::Missing)),
            Some(key) => {
                if is_valid(key) {
                    Outcome::Success(PhotoKey(key))
                } else {
                    Outcome::Error((Status::Forbidden, PhotoKeyError::Negative))
                }
            }
        }
    }
}

#[get("/")]
fn hello() -> &'static str {
    "Hello, world!"
}

#[post("/divide", format = "json", data = "<factors>")]
fn divide(factors: Json<Factors>) -> String {
    if factors.divisor != 0. {
        (factors.dividend / factors.divisor).to_string()
    } else {
        "cannot divide by 0".to_string()
    }
}

#[post("/message", format = "json", data = "<message>")]
fn set_message(message: Json<Message>, message_state: &State<Message>) {
    let mut message_state_guard = message_state.body.lock().unwrap();
    *message_state_guard = message.body.lock().unwrap().clone();
}

#[get("/message")]
fn get_message(message_state: &State<Message>) -> String {
    message_state.body.lock().unwrap().to_string()
}

#[get("/photo")]
async fn get_photo(_photo_key: PhotoKey<'_>) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/img/nice_photo.JPG"))
        .await
        .ok()
}

#[get("/script?<a>&<b>")]
fn script(a: &str, b: &str) -> String {
    // Command::new("python3").args([a, b]).output()
    let res: String = match Command::new("python3")
        .args(["./static/py/sum.py", a, b])
        .output()
    {
        Ok(out) => match str::from_utf8(&out.stdout) {
            Ok(text) => text.trim().to_string(),
            Err(e) => return format!("couldn't read the script output: {}", e).to_string(),
        },
        Err(e) => return format!("the script couldn't be executed: {}", e).to_string(),
    };
    res
}

#[get("/users")]
async fn get_users(users_col: &State<UsersCol>) -> Json<Vec<User>> {
    let users = get_user_query(users_col).await;
    match users {
        Ok(users_vector) => Json(users_vector),
        Err(err) => Json(vec![User {
            _id: ObjectId::new(),
            name: err.to_string(),
            contacts: vec!["Error".to_string()],
            password: "Error".to_string(),
        }]),
    }
}

#[post("/user", format = "json", data = "<user>")]
async fn add_user(user: Json<User>, users_col: &State<UsersCol>) -> String {
    let result = add_user_query(users_col, user.into_inner()).await;
    match result {
        Ok(res) => res.inserted_id.to_string(),
        Err(err) => format!("no document inserted: {}", err),
    }
}

#[post("/delete_user", format = "json", data = "<user_name>")]
async fn delete_user(user_name: Json<DeleteFilter>, users_col: &State<UsersCol>) -> String {
    let result = delete_user_query(users_col, user_name.into_inner().name).await;
    match result {
        Ok(res) => format!("{} user deleted", res.deleted_count),
        Err(err) => format!("no document deleted: {}", err),
    }
}

//this is run as main
#[launch]
async fn rocket() -> _ {
    dotenv().ok(); // loads environment variables from .env file
    let mongo_client = mdb_connection().await.expect("couldn't connect to MongoDB");

    rocket::build()
        .manage(Message {
            body: Mutex::new("".to_string()),
        })
        .manage(UsersCol {
            col: mongo_client.database("decisionFlow").collection("users"),
        })
        // routes defined above: get
        .mount("/", routes![hello])
        // post
        .mount("/", routes![divide])
        // route used to serve static files
        .mount("/index", FileServer::from(relative!("static")))
        .mount("/style", FileServer::from(relative!("static/css")))
        .mount("/js", FileServer::from(relative!("static/js")))
        // route to set state
        .mount("/", routes![set_message])
        // route to get state
        .mount("/", routes![get_message])
        // route that uses custom guard
        .mount("/", routes![get_photo])
        // route that execute an external script
        .mount("/", routes![script])
        // route that query the database for the users
        .mount("/", routes![get_users])
        // route that query the database to insert a new user
        .mount("/", routes![add_user])
        // route that query the database to delete a user
        .mount("/", routes![delete_user])
}

async fn mdb_connection() -> Result<Client, Box<dyn Error>> {
    // Load the MongoDB connection string from an environment variable:
    let client_uri =
        env::var("MONGODB_URI").expect("You must set the MONGODB_URI environment var!");

    // A Client is needed to connect to MongoDB:
    // An extra line of code to work around a DNS issue on Windows:
    let options =
        ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare())
            .await?;

    let client = Client::with_options(options)?;

    Ok(client)
}

async fn get_user_query(users_col: &State<UsersCol>) -> Result<Vec<User>, Box<dyn Error>> {
    let mut result: Vec<User> = vec![];
    let mut users = users_col.col.find(doc! {}, None).await?;
    while users.advance().await? {
        result.push(users.deserialize_current().unwrap());
    }
    Ok(result)
}

async fn add_user_query(
    users_col: &State<UsersCol>,
    new_user: User,
) -> Result<InsertOneResult, Box<dyn Error>> {
    let users = users_col.col.insert_one(new_user, None).await?;
    Ok(users)
}

async fn delete_user_query(
    users_col: &State<UsersCol>,
    user_name: String,
) -> Result<DeleteResult, Box<dyn Error>> {
    let users = users_col
        .col
        .delete_one(doc! {"name": user_name}, None)
        .await?;
    Ok(users)
}

#[cfg(test)]
mod test {
    use super::rocket;
    use bson::oid::ObjectId;
    use rocket::http::{ContentType, Header, Status};
    use rocket::local::asynchronous::Client;
    use rocket::serde::json;

    #[async_test]
    async fn hello_test() {
        let client = Client::tracked(rocket().await)
            .await
            .expect("valid rocket instance");
        let response = client.get(uri!(super::hello)).dispatch().await;
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().await.unwrap(), "Hello, world!");
    }

    #[async_test]
    async fn index_test() {
        let client = Client::tracked(rocket().await)
            .await
            .expect("valid rocket instance");
        let response = client.get(uri!("/index")).dispatch().await;
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type().unwrap(), ContentType::HTML);
        // could use some other tests here but I don't know what to test
    }

    #[async_test]
    async fn divide_test() {
        let client = Client::tracked(rocket().await)
            .await
            .expect("valid rocket instance");
        let response = client
            .post(uri!("/divide"))
            .header(ContentType::JSON)
            .body(r#"{"dividend": 4, "divisor": 2}"#)
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type().unwrap(), ContentType::Text);
        assert_eq!(response.into_string().await.unwrap(), "2");
    }

    #[async_test]
    async fn message_test() {
        async fn set_and_read(message_name: &str) {
            let client = Client::tracked(rocket().await)
                .await
                .expect("valid rocket instance");

            let get_response = client.get(uri!("/message")).dispatch().await;
            assert_eq!(get_response.status(), Status::Ok);
            assert_eq!(get_response.content_type().unwrap(), ContentType::Text);
            assert_eq!(get_response.into_string().await.unwrap(), "");

            let set_response = client
                .post(uri!("/message"))
                .header(ContentType::JSON)
                .body(json::to_string(&json::json!({"body": message_name})).unwrap())
                .dispatch()
                .await;
            assert_eq!(set_response.status(), Status::Ok);

            let get_response = client.get(uri!("/message")).dispatch().await;
            assert_eq!(get_response.status(), Status::Ok);
            assert_eq!(get_response.content_type().unwrap(), ContentType::Text);
            assert_eq!(get_response.into_string().await.unwrap(), message_name);
        }

        set_and_read("Sirio").await;
        set_and_read("Andromeda").await;
        set_and_read("Altair").await;
        set_and_read("Polluce").await;
    }

    #[async_test]
    async fn req_guards_test() {
        let client = Client::tracked(rocket().await)
            .await
            .expect("valid rocket instance");
        let response = client
            .get(uri!("/photo"))
            .header(Header::new("nice-photo", "yes"))
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type().unwrap(), ContentType::JPEG);

        let response = client
            .get(uri!("/photo"))
            .header(Header::new("nice-photo", "no"))
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Forbidden);

        let response = client.get(uri!("/photo")).dispatch().await;
        assert_eq!(response.status(), Status::BadRequest);
    }

    #[async_test]
    async fn script_test() {
        let client = Client::tracked(rocket().await)
            .await
            .expect("valid rocket instance");
        let response = client
            .get(uri!("/script?a=9182.467&b=3057"))
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type().unwrap(), ContentType::Text);
        assert_eq!(response.into_string().await.unwrap(), "12239.467");

        let client = Client::tracked(rocket().await)
            .await
            .expect("valid rocket instance");
        let response = client.get(uri!("/script?a=9182.467")).dispatch().await;
        assert_eq!(response.status(), Status::UnprocessableEntity);

        let client = Client::tracked(rocket().await)
            .await
            .expect("valid rocket instance");
        let response = client
            .get(uri!("/script?a=9182.467&b=vega"))
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type().unwrap(), ContentType::Text);
    }

    #[async_test]
    async fn mongodb_read_test() {
        let client = Client::tracked(rocket().await)
            .await
            .expect("valid rocket instance");
        let response = client.get(uri!("/users")).dispatch().await;
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type().unwrap(), ContentType::JSON);
    }

    #[async_test]
    async fn mongodb_write_and_delete_test() {
        let client = Client::tracked(rocket().await)
            .await
            .expect("valid rocket instance");
        let response = client
            .post(uri!("/user"))
            .header(ContentType::JSON)
            .body(
                json::to_string(
                    &json::json!({"_id":ObjectId::new(),"name":"test_user","contacts":["test@gtest.com"],"password":""}),
                )
                .unwrap(),
            )
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type().unwrap(), ContentType::Text);

        let response = client
            .post(uri!("/delete_user"))
            .header(ContentType::JSON)
            .body(json::to_string(&json::json!({"name":"test_user"})).unwrap())
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type().unwrap(), ContentType::Text);
    }
}
