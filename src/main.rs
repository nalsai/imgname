use exif::DateTime;
use filetime::FileTime;
use same_file::is_same_file;
use std::fs;
use std::path::Path;
use std::time::{Duration, UNIX_EPOCH};

mod cli;

fn main() {
    let matches = cli::build_cli().get_matches();

    let filetime = match matches.get_one::<bool>("filetime") {
        Some(val) if val == &true => true,
        _ => false,
    };

    match matches.subcommand() {
        Some((command, sub_matches)) => {
            for path in sub_matches.get_many::<String>("PATH").into_iter().flatten() {
                match handle_file(command, path, &filetime) {
                    Ok(_) => (),
                    Err(e) => println!("{} {:?}", path, e),
                }
            }
        }
        _ => unreachable!(),
    }
}

fn handle_file(command: &str, path: &str, filetime: &bool) -> Result<(), exif::Error> {
    match command {
        "rename" | "move" | "rename-move" => {
            let datetime = match filetime {
                false => get_datetime(path)?,
                true => get_filedatetime(path)?,
            };

            move_file(command, path, datetime)?;
        }
        "get-date" => {
            name_to_date_helper(path);
        }
        "get-name" => {
            date_to_name_helper(path)?;
        }
        _ => unreachable!(),
    }
    Ok(())
}

fn get_datetime(path: &str) -> Result<DateTime, exif::Error> {
    let file = std::fs::File::open(&path)?;
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();

    let exif = exifreader.read_from_container(&mut bufreader)?;

    let field = match exif.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY) {
        Some(field) => field,
        None => return Err(exif::Error::BlankValue("File has no DateTimeOriginal")),
    };

    match field.value {
        exif::Value::Ascii(ref vec) if !vec.is_empty() => {
            if let Ok(dt) = DateTime::from_ascii(&vec[0]) {
                return Ok(dt);
            }
        }
        _ => return Err(exif::Error::BlankValue("DateTime is not readable")),
    }
    return Err(exif::Error::BlankValue("DateTime is not readable"));
}

fn get_filedatetime(path: &str) -> Result<DateTime, exif::Error> {
    let metadata = fs::metadata(path)?;

    let mtime = FileTime::from_last_modification_time(&metadata);
    let datetime = chrono::DateTime::<chrono::Local>::from(
        UNIX_EPOCH + Duration::from_secs(mtime.unix_seconds() as u64),
    );

    let timestamp_str = datetime.format("%Y:%m:%d %H:%M:%S").to_string();
    return Ok(DateTime::from_ascii(timestamp_str.as_bytes())?);
}

/* Compare if a file path1.a is equal to path2.a and path1.b to path2.b */
fn files_with_same_extension_are_equal(path1: &Path, path2: &Path) -> bool {

    if !(path1.exists() && path2.exists()) {
        return false;
    }

    let path1_ext = path1.extension().unwrap().to_str().unwrap();
    let path2_ext = path2.extension().unwrap().to_str().unwrap();

    let path1_with_ext2 = path1.with_extension(&path2_ext);
    let path2_with_ext1 = path2.with_extension(&path1_ext);
    
    if !(path1_with_ext2.exists() && path2_with_ext1.exists()) {
        return false;
    }

    return is_same_file(&path1, &path2_with_ext1).unwrap()
        && is_same_file(&path2, path1_with_ext2).unwrap()
}

fn move_file(method: &str, src_path: &str, mut datetime: DateTime) -> Result<(), exif::Error> {
    let src_filepath = Path::new(&src_path);
    let src_parent = src_filepath.parent().unwrap();    // can be empty
    let src_dir = if src_parent.as_os_str().is_empty() { Path::new(".") } else { src_parent }; // if src_parent is empty, the current directory is used
    let src_name_stem = src_filepath.file_stem().unwrap();

    let dest_subdir = if method == "move" || method == "rename-move" { date_to_directory(&datetime) } else { String::default() };
    let dest_name = if method == "rename" || method == "rename-move" { date_to_name(&datetime) } else { Path::new(src_path).file_stem().unwrap().to_string_lossy().to_string() };
    let dest_dir = &src_dir.join(&dest_subdir);

    let mut dest_name_unique: String;

    // make dest_name_stem unique in both src_dir and dest_dir    
    let mut counter: u8 = 1;
    (dest_name_unique, counter, datetime) = make_unique_filename(src_dir, &dest_name, dest_name.clone(), datetime, src_filepath, counter, method);
    println!("->{}", dest_name_unique);

    if !dest_subdir.is_empty() && dest_dir.exists() { // !dest_subdir.is_empty() means that dest_dir != src_dir
        (dest_name_unique, _, _) = make_unique_filename(dest_dir, &dest_name, dest_name_unique, datetime, src_filepath, counter, method);
    }
    println!("->{}", dest_name_unique);

    // find and move all files with src_name_stem in src_dir to dest_dir with dest_name_unique
    for entry in fs::read_dir(src_dir)? {
        match entry {
            Ok(entry) => {

                // if the file name matches src_name, move it to dest_dir with dest_name
                let entry_path = entry.path();
                if entry_path.file_stem().unwrap() == src_name_stem {
                    let dest_ext = entry_path.extension().unwrap().to_str().unwrap().to_ascii_lowercase();
                    let dest_path = dest_dir.join(&dest_name_unique).with_extension(&dest_ext);

                    println!("{} -> {}", entry_path.to_str().unwrap_or_default(), dest_path.to_str().unwrap_or_default());
                    fs::create_dir_all(&dest_dir).unwrap_or_default();

                    // make absolutely sure no file will be overwritten
                    if dest_path.exists() && !is_same_file(entry.path(), &dest_path).unwrap(){
                        panic!("Error: trying to move {} to {} but a file already exists at the target", entry.path().to_str().unwrap_or_default(), dest_path.to_str().unwrap_or_default());
                    }
                    fs::rename(entry.path(), dest_path)?;
                }
            },
            Err(_) => {
                println!("Error reading entry");
                continue
            },
        }
    }

    Ok(())
}

fn make_unique_filename(path: &Path, dest_name: &str, mut dest_name_unique: String, mut datetime: DateTime, src_filepath: &Path, counter: u8, method: &str) -> (String, u8, DateTime) {
    let mut counter = counter;
    // TODO: make the name check work even if the files are out of order (important)
    for entry in fs::read_dir(path).unwrap() { // TODO: handle error
        match entry {
            Ok(entry) => {
                let entry_path = entry.path();

                if entry_path.file_stem().unwrap().to_str().unwrap() == dest_name_unique {
                    if !files_with_same_extension_are_equal(&src_filepath, &entry_path) {
                        println!("{} already exists", entry_path.to_str().unwrap_or_default());
                        if method == "rename" || method == "rename-move"{
                            datetime.second += 1;
                            dest_name_unique = date_to_name(&datetime);
                        } else {
                            dest_name_unique = format!("{}-{}", dest_name, counter);
                            counter += 1;
                        }
                    }
                }
            },
            Err(_) => {
                println!("Error reading entry");
                continue
            },
        }
    }

    (dest_name_unique, counter, datetime)
}

fn date_to_directory(datetime: &DateTime) -> String {
    return format!(
        "{:04}-{:02}-{:02}",
        datetime.year, datetime.month, datetime.day
    );
}

fn u8_to_char(num: u8, uppercase: bool) -> char {
    return (num + if uppercase { 64 } else { 96 }) as char;
}

fn char_to_u8(letter: char) -> Option<u8> {
    let c = letter.to_ascii_uppercase() as u8;
    if c <= 64 {
        return None;
    }
    return Some(c - 64);
}

fn date_to_name_helper(date_string: &str) -> Result<(), exif::Error> {
    let dt_bytes = date_string.as_bytes();
    let dt = DateTime::from_ascii(dt_bytes)?;
    let name = date_to_name(&dt);
    println!("{} -> {}", &dt, &name);
    Ok(())
}

fn date_to_name(dt: &DateTime) -> String {
    let mut name = String::default();
    name.push(u8_to_char((dt.year - 1999) as u8, true));
    name.push(u8_to_char(dt.month, true));
    if dt.day <= 9 {
        name += &dt.day.to_string();
    } else {
        name.push(u8_to_char(dt.day - 9, true));
    }
    name.push_str(&format!(
        "{:0>5}",
        &(dt.hour as u32 * 3600 + dt.minute as u32 * 60 + dt.second as u32).to_string()
    ));

    return name;
}

fn name_to_date_helper(path: &str) {
    let name = Path::new(&path)
        .file_stem()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default();

    let clean_name = name.replace("VID_", "");
    if clean_name.len() < 8 {
        println!("{} is not in the correct format", name);
        return;
    }
    let cleaner_name = &clean_name[..8];

    match name_to_date(&cleaner_name) {
        Some(dt) => println!("{} -> {}", name, &dt),
        None => println!("{} is not in the correct format", name),
    }
}

fn name_to_date(name: &str) -> Option<DateTime> {
    let mut chars = name.chars();

    let year = char_to_u8(chars.next()?)? as u16 + 1999;
    let month = char_to_u8(chars.next()?)?;
    let day_char = chars.next()?;
    let day = if day_char.is_digit(10) {
        day_char.to_digit(10)? as u8
    } else {
        char_to_u8(day_char)? + 9
    };

    let time_string = chars.collect::<String>();

    match time_string.parse::<u32>() {
        Ok(time_string) => {
            let time_value = time_string;

            let hour = time_value / 3600;
            let minute = (time_value - hour * 3600) / 60;
            let second = time_value - hour * 3600 - minute * 60;

            return Some(DateTime {
                year: year,
                month: month,
                day: day,
                hour: hour as u8,
                minute: minute as u8,
                second: second as u8,
                nanosecond: Option::default(),
                offset: Option::default(),
            });
        }
        Err(e) => {
            print!("{}: ", e)
        }
    }

    return None;
}
