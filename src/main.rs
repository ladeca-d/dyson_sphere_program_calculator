use futures::stream::TryStreamExt;
use glob::glob;
use percent_encoding::percent_decode;
use std::fs;
use std::io::prelude::*;
use std::path::Path;
use warp::{http::Response, Buf, Filter};

const LOADSTR: &str = r#"<option value="%forbidden%">content</option>"#;

#[tokio::main]
async fn main() {
    let default_reply = warp::path::end()
        .and(warp::get())
        .map(|| Response::builder().body(default_reply()));

    let post_reply = warp::post()
        .and(warp::multipart::form())
        .and_then(post_reply);

    let routers = warp::get().and(default_reply).or(post_reply);

    warp::serve(routers).run(([127, 0, 0, 1], 3030)).await;
}

async fn post_reply(form: warp::multipart::FormData) -> Result<impl warp::Reply, warp::Rejection> {
    let mut parts: Vec<warp::multipart::Part> = form
        .try_collect()
        .await
        .map_err(|_e| warp::reject::reject())?;

    if parts[0].name() == "save" {
        let buf = parts[0].data().await.unwrap().unwrap();
        let file_name = percent_decode(&buf.chunk()[..])
            .decode_utf8()
            .unwrap()
            .to_string();

        let mut ew = None;
        let mut yy = None;
        for part in parts[1..].iter_mut() {
            let key = part.name().to_string();
            let buf = part.data().await.unwrap().unwrap();
            let value = percent_decode(&buf.chunk()[..])
                .decode_utf8()
                .unwrap()
                .to_string();

            if key == "ew" {
                ew = Some(value);
            } else if key == "yy" {
                yy = Some(value);
            }
        }

        if ew.is_some() && yy.is_some() {
            record(file_name, ew.unwrap().to_string(), yy.unwrap().to_string());
        }

        Ok(Response::builder().body("OK".to_string()))
    } else if parts[0].name() == "load" {
        let buf = parts[0].data().await.unwrap().unwrap();
        let file_name = percent_decode(&buf.chunk()[..])
            .decode_utf8()
            .unwrap()
            .to_string();

        let path_string = "./".to_string() + &file_name + &".ini";
        let contents = fs::read_to_string(path_string).unwrap();

        Ok(Response::builder().body(contents))
    } else {
        unreachable!("错误的POST请求，请联系作者修复！")
    }
}

fn record(name: String, ew: String, yy: String) {
    let name: String = name + ".ini";
    let path_string = "./".to_string() + &name;
    if Path::new(&path_string).exists() {
        fs::remove_file(&path_string).unwrap();
    }

    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path_string)
        .unwrap();

    let content = ew + &"\r\n" + &yy[..];
    file.write_all(content.as_bytes()).unwrap();
}

fn default_reply() -> String {
    let name = "index.html";
    let path_string = "./".to_string() + &name;

    let contents = fs::read_to_string(path_string).unwrap();

    show_load_files(contents)
}

fn show_load_files(contents: String) -> String {
    let mut path_vec = vec![];
    for entry in glob("./*.ini").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => path_vec.push(path.into_os_string().into_string().unwrap()),
            Err(e) => println!("{:?}", e),
        }
    }

    let mut new_load_files = "".to_string();
    for path in path_vec.iter() {
        new_load_files = new_load_files
            + LOADSTR
                .replace("%forbidden%", &path[..path.len() - 4])
                .replace("content", &path[..path.len() - 4])
                .as_str();
    }
    contents.replace(LOADSTR, new_load_files.as_str())
}
