#![feature(plugin)]
#![plugin(rocket_codegen)]
use rocket::response::NamedFile;
use std::io::Read;
use rocket::{Request, Data};
use rocket::data::{self, FromData};
use rocket::http::{Status};
use rocket::Outcome::*;

extern crate rocket;

fn hex_to_u8(chr: u8) -> u8 {
    if chr >= 48 && chr <= 57 {
        chr as u8 - 48
    } else {
        chr as u8 - 55
    }
}

fn from_url(url: &str) -> String {
    let fragments : Vec<_> = url.split("%").collect();
    let mut result = String::new();
    let mut first = true; // first time through is not because of '%'
    for s in fragments {
        if first {
            first = false;
            result.push_str(s);
            continue;
        }
        let sb = s.as_bytes();
        if sb[0] == '%' as u8 {
            result.push_str(s);
        } else {
            let chr = (hex_to_u8(sb[0]) << 4 | hex_to_u8(sb[1])) as char;
            result.push(chr);
            result.push_str(&s[2..]);
        }
    }
    result
}

struct Person {
    name: String,
    age: u8
}

impl FromData for Person {
    type Error = String;

    fn from_data(_: &Request, data: Data) -> data::Outcome<Self, String> {
        let mut string = String::new();
        if let Err(e) = data.open().read_to_string(&mut string) {
            return Failure((Status::InternalServerError, format!("{:?}", e)));
        }
        println!("{}", string);
        let param: &str = &string;
        let split: &Vec<_> = &param[1..].split("&").collect();
        let name = match split[0].find('=') {
            Some(i) => from_url(&(split[0])[i+1..]),
            _ => return Failure((Status::BadRequest, "Bad name".into()))
        };
        let age = match split[1].find('=') {
            Some(i) => (&(split[1])[i+1..]).to_string().parse::<u8>()
            .map(|n| n)
            .map_err(|e| e),
            _ => return Failure((Status::BadRequest, "Bad age".into()))
        };
        let age = match age {
            Ok(n) => n,
            Err(_) => return Failure((Status::BadRequest, "Bad age".into()))
        };
        Success(Person {name: name, age: age})
    }
}

#[post("/Return.html", data = "<person>")]
fn hello(person: Person) -> String {
    format!("{} is {} years old", person.name, person.age)
}

#[get("/")]
fn home() -> Option<NamedFile> {
    NamedFile::open("html/home.html").ok()
}

#[get("/<file>")]
fn static_file(file: String) -> Option<NamedFile> {
    NamedFile::open("html/".to_string() + &file).ok()
}

fn main() {
    rocket::ignite().mount("/", routes![hello, home, static_file]).launch();
}
