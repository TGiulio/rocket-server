#[macro_use]
extern crate rocket;

use rocket::fs::{relative, FileServer};
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use std::str;

#[derive(Deserialize)]
struct Factors {
    dividend: f32,
    divisor: f32,
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

//this is run as main
#[launch]
fn rocket() -> _ {
    rocket::build()
        // routes defined above: get
        .mount("/", routes![hello])
        // post
        .mount("/", routes![divide])
        // route used to serve static files
        .mount("/index", FileServer::from(relative!("static")))
        .mount("/style", FileServer::from(relative!("static/css")))
        .mount("/js", FileServer::from(relative!("static/js")))
}

#[cfg(test)]
mod test {
    use super::rocket;
    use rocket::http::{ContentType, Status};
    use rocket::local::blocking::Client;

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
}
