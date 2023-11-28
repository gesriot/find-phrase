use std::env;
use std::fs::{self, File};
use std::io::{self, BufReader, Read};
use std::path::Path;
use encoding_rs::*;
use chardet::{charset2encoding, detect};
use rayon::prelude::*;
use std::io::BufRead;


fn read_file_to_vec(file_path: &str) -> io::Result<Vec<String>> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let detector_result = detect(&buffer);
    let encoding_name = charset2encoding(&detector_result.0);
    let encoding = Encoding::for_label(encoding_name.as_bytes()).unwrap();

    let (decoded_buffer, _, _) = encoding.decode(&buffer);

    let lines = decoded_buffer.lines().map(|l| l.to_string()).collect();
    Ok(lines)
}

fn search_phrase_in_file(file_path: &Path, phrase: &str) -> io::Result<()> {
    let lines = read_file_to_vec(file_path.to_str().unwrap())?;
    let mut found = false;
    for (i, line) in lines.iter().enumerate() {
        if line.contains(phrase) {
            if !found {
                println!("Файл: {}", file_path.display());
                found = true;
            }
            println!("Строка {}: {}", i + 1, line);
        }
    }
    if found {
        println!();
    }
    Ok(())
}

fn search_phrase_in_files(dir_path: &Path, file_extension: &str, phrase: &str) -> io::Result<()> {
    if dir_path.is_dir() {
        let entries: Vec<_> = fs::read_dir(dir_path)?.collect();
        entries.into_par_iter().for_each(|entry| {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                search_phrase_in_files(&path, file_extension, phrase).unwrap();
            } else if path.extension().and_then(|s| s.to_str()) == Some(file_extension) {
                search_phrase_in_file(&path, phrase).unwrap();
            }
        });
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let (dir_path, file_extension, phrase) = if args.len() == 4 {
        (args[1].clone(), args[2].clone(), args[3].clone())
    } else if args.len() == 1 {
        let config = File::open("config.txt")?;
        let mut lines = BufReader::new(config).lines();
        let dir_path = lines.next().unwrap_or(Ok(String::new()))?;
        let file_extension = lines.next().unwrap_or(Ok(String::new()))?;
        let phrase = lines.next().unwrap_or(Ok(String::new()))?;
        (dir_path, file_extension, phrase)
    } else {
        println!("Используй: {} <путь к директории> <расширение файла> <искомая фраза>", args[0]);
        return Ok(());
    };
    let dir_path = Path::new(&dir_path);
    if !dir_path.exists() {
        println!("Директории {} не существует.", dir_path.display());
        return Ok(());
    }
    search_phrase_in_files(dir_path, &file_extension, &phrase)?;

    println!("Нажмите Enter для выхода...");
    let mut pause = String::new();
    io::stdin().read_line(&mut pause)?;

    Ok(())
}
