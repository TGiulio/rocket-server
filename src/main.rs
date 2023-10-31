#[macro_use]
extern crate rocket;

use rocket::fs::{relative, FileServer};

#[get("/")]
fn hello() -> &'static str {
    "Hello, world!"
}

//this is run as main
#[launch]
fn rocket() -> _ {
    rocket::build()
        // route defined above
        .mount("/", routes![hello])
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
}
