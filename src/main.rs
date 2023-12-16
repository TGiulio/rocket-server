#[macro_use]
extern crate rocket;

use rocket::fs::{relative, FileServer, NamedFile};
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::State;

use std::path::Path;
use std::str;
use std::sync::Mutex;
struct PhotoKey<'r>(&'r str);

#[derive(Deserialize)]
struct User {
    username: Mutex<String>,
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

#[post("/username", format = "json", data = "<user>")]
fn set_username(user: Json<User>, user_state: &State<User>) {
    let mut username_state_guard = user_state.username.lock().unwrap();
    *username_state_guard = user.username.lock().unwrap().clone();
}

#[get("/username")]
fn get_username(user_state: &State<User>) -> String {
    user_state.username.lock().unwrap().to_string()
}

#[get("/photo")]
async fn get_photo(_photo_key: PhotoKey<'_>) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/img/nice_photo.JPG"))
        .await
        .ok()
}

//this is run as main
#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(User {
            username: Mutex::new("".to_string()),
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
        .mount("/", routes![set_username])
        // route to get state
        .mount("/", routes![get_username])
        // route that uses custom guard
        .mount("/", routes![get_photo])
}

#[cfg(test)]
mod test {
    use super::rocket;
    use rocket::http::{ContentType, Header, Status};
    use rocket::local::blocking::Client;
    use rocket::serde::json;

    #[test]
    fn hello_test() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get(uri!(super::hello)).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().unwrap(), "Hello, world!");
    }

    #[test]
    fn index_test() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get(uri!("/index")).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type().unwrap(), ContentType::HTML);
        // could use some other tests here but I don't know what to test
    }

    #[test]
    fn divide_test() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client
            .post(uri!("/divide"))
            .header(ContentType::JSON)
            .body(r#"{"dividend": 4, "divisor": 2}"#)
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type().unwrap(), ContentType::Text);
        assert_eq!(response.into_string().unwrap(), "2");
    }

    #[test]
    fn username_test() {
        fn set_and_read(user_name: &str) {
            let client = Client::tracked(rocket()).expect("valid rocket instance");

            let get_response = client.get(uri!("/username")).dispatch();
            assert_eq!(get_response.status(), Status::Ok);
            assert_eq!(get_response.content_type().unwrap(), ContentType::Text);
            assert_eq!(get_response.into_string().unwrap(), "");

            let set_response = client
                .post(uri!("/username"))
                .header(ContentType::JSON)
                .body(json::to_string(&json::json!({"username": user_name})).unwrap())
                .dispatch();
            assert_eq!(set_response.status(), Status::Ok);

            let get_response = client.get(uri!("/username")).dispatch();
            assert_eq!(get_response.status(), Status::Ok);
            assert_eq!(get_response.content_type().unwrap(), ContentType::Text);
            assert_eq!(get_response.into_string().unwrap(), user_name);
        }

        set_and_read("Sirio");
        set_and_read("Andromeda");
        set_and_read("Altair");
        set_and_read("Polluce");
    }

    #[test]
    fn req_guards_test() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client
            .get(uri!("/photo"))
            .header(Header::new("nice-photo", "yes"))
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type().unwrap(), ContentType::JPEG);

        let response = client
            .get(uri!("/photo"))
            .header(Header::new("nice-photo", "no"))
            .dispatch();
        assert_eq!(response.status(), Status::Forbidden);

        let response = client.get(uri!("/photo")).dispatch();
        assert_eq!(response.status(), Status::BadRequest);
    }
}
