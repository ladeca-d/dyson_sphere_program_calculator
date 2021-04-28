use percent_encoding::percent_decode;
use glob::glob;
use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::path::Path;

const ELEMN: usize = 82;
const EWSTR: &str = r#"name="ew" size="1%" value="0""#;
const YYSTR: &str = r#"name="yy" size="1%" value="0""#;
const LOADSTR: &str = r#"<option value="%forbidden%">content</option>"#;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:3030").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024 * 2];
    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let post = b"POST / HTTP/1.1\r\n";

    if buffer.starts_with(get) {
        let status_line = "HTTP/1.1 200 OK\r\n\r\n";
        let name = "index.html";
        let path_string = "./".to_string() + &name;

        let mut contents = fs::read_to_string(path_string).unwrap();
        contents = show_load_files(contents);
        let response = format!("{}{}", status_line, contents);

        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    } else if buffer.starts_with(post) {
        let header = percent_decode(&buffer[..]).decode_utf8().unwrap();
        let header = header.split("\n").collect::<Vec<&str>>();
        let content = header[header.len() - 1];
        let content_vec = content.split("&").collect::<Vec<&str>>();

        let name = content_vec[0].split("=").collect::<Vec<&str>>()[1];

        if content_vec[0].starts_with("save") {
            let mut ew = ["0"; ELEMN];
            let mut yy = ["0"; ELEMN];
            for (idx, elem) in content_vec[2..ELEMN * 2 + 2 + 1].iter().enumerate() {
                let elem_vec = elem.split("=").collect::<Vec<&str>>();
                if idx / ELEMN == 0 {
                    ew[idx] = elem_vec[1];
                } else if idx != ELEMN {
                    let idx2 = idx - 1;
                    yy[idx2 % ELEMN] = elem_vec[1];
                }
            }

            record(name, ew, yy);

            // renew code
            let status_line = "HTTP/1.1 200 OK\r\n\r\n";
            let name = "index.html";
            let path_string = "./".to_string() + &name;
    
            let mut contents = fs::read_to_string(path_string).unwrap();
            contents = show_load_files(contents);
            let response = format!("{}{}", status_line, contents);
    
            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        } else if content_vec[0].starts_with("load") {
            let name: String = name.to_string() + ".ini";
            let path_string = "./".to_string() + &name;

            let contents = fs::read_to_string(path_string).unwrap();
            let contents = contents.split(",").collect::<Vec<&str>>();
        
            let mut ew = ["0"; ELEMN];
            let mut yy = ["0"; ELEMN];
            if contents.len() != ELEMN * 2 {
                println!("申请读取一份无效的或者被错误修改过的预设文件。");
            } else {
                for (idx, elem) in contents.iter().enumerate() {
                    if idx / ELEMN == 0 {
                        ew[idx] = elem;
                    } else {
                        yy[idx % ELEMN] = elem;
                    }
                }
            }

            let status_line = "HTTP/1.1 200 OK\r\n\r\n";
            let name = "index.html";
            let path_string = "./".to_string() + &name;
    
            let mut contents = fs::read_to_string(path_string).unwrap();
            contents = show_load_files(contents);
            
            // modify contents
            for i in 0..ELEMN {
                // replace the value
                if ew[i] != "0" {
                    let new_ew = EWSTR.replace("0", ew[i]);
                    contents = contents.replacen(EWSTR, new_ew.as_str(), 1);
                } else {
                    let new_ew = EWSTR.replace("0", "-1");
                    contents = contents.replacen(EWSTR, new_ew.as_str(), 1);
                }
                
                if yy[i] != "0" {
                    let new_yy = YYSTR.replace("0", yy[i]);
                    contents = contents.replacen(YYSTR, new_yy.as_str(), 1);
                } else {
                    let new_yy = YYSTR.replace("0", "-1");
                    contents = contents.replacen(YYSTR, new_yy.as_str(), 1);
                }
            }

            let new_ew = EWSTR.replace("0", "-1");
            contents = contents.replace(new_ew.as_str(), EWSTR);
            let new_yy = YYSTR.replace("0", "-1");
            contents = contents.replace(new_yy.as_str(), YYSTR);


            let response = format!("{}{}", status_line, contents);

            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
    }
}

fn record(name: &str, ew: [&str; ELEMN], yy: [&str; ELEMN]) {
    let name: String = name.to_string() + ".ini";
    let path_string = "./".to_string() + &name;
    if Path::new(&path_string).exists() {
        fs::remove_file(&path_string).unwrap();
    }

    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path_string)
        .unwrap();

    let mut content = ew[0].to_string();
    for elem in ew[1..].iter().chain(yy.iter()) {
        content = content + &"," + elem;
    }

    file.write_all(content.as_bytes()).unwrap();
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
        new_load_files = new_load_files + LOADSTR.replace("%forbidden%",  &path[..path.len() - 4]).replace("content", &path[..path.len() - 4]).as_str();
    }
    contents.replace(LOADSTR, new_load_files.as_str())
}