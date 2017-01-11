#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket_contrib;
extern crate rocket;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate rand;

mod word_lists;

use std::io;
use std::fs::File;
use rocket::response::NamedFile;
use rocket::request::Form;
use rocket_contrib::Template;
use rocket::response::Redirect;
use std::io::Write;
use std::io::Read;
use rand::Rng;
use std::path::{Path, PathBuf};

const BASE62: &'static [u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

#[derive(Serialize)]
struct TemplateContext {
    feminine_words: Vec<String>,
    masculine_words: Vec<String>,
    ad_text: String,
}

#[derive(FromForm)]
struct Ad {
    ad_text: String,
}

fn ad_decoder(ad: &str, word_list: Vec<&str>) -> Vec<String> {
    let mut results = vec![];
    let ad_words = ad.split(|s: char| !s.is_alphabetic())
        .map(|s| s.to_lowercase());

    for ad_word in ad_words {
        if word_list.iter().any(|word| ad_word.starts_with(word)) {
            results.push(ad_word.to_string());
        }
    }

    results
}

fn get_id() -> String {
    let size = 10;
    let mut id = String::with_capacity(size);
    let mut rng = rand::thread_rng();

    for _ in 0..size {
        id.push(BASE62[rng.gen::<usize>() % 62] as char);
    }

    id
}

#[get("/")]
fn index() -> Template {
    let mut map = std::collections::HashMap::new();
    map.insert("index", true);
    Template::render("index", &map)
}

#[get("/<id>")]
fn get_by_id(id: String) -> io::Result<Template> {
    let mut ad_text = String::new();

    File::open(Path::new(&format!("uploads/{id}", id = id)))
        .and_then(|mut s| s.read_to_string(&mut ad_text))?;

    let feminine_results = ad_decoder(&ad_text, word_lists::FEMININE_WORDS.to_vec());
    let masculine_results = ad_decoder(&ad_text, word_lists::MASCULINE_WORDS.to_vec());

    let context = TemplateContext {
        feminine_words: feminine_results,
        masculine_words: masculine_results,
        ad_text: ad_text,
    };

    Ok(Template::render("index", &context))
}

#[post("/save", data = "<ad_form>")]
fn save(ad_form: Form<Ad>) -> io::Result<Redirect> {
    let id = get_id();
    let path = format!("uploads/{id}", id = id);
    let ad_text = &ad_form.get().ad_text;

    File::create(Path::new(&path))
        .and_then(|mut file| file.write_all(ad_text.as_bytes()))?;

    Ok(Redirect::to(&format!("/{id}", id = id)))
}

#[get("/<path..>", rank = 5)]
fn static_files(path: PathBuf) -> io::Result<NamedFile> {
    NamedFile::open(Path::new("static/").join(path))
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, save, get_by_id, static_files])
        .launch();
}
