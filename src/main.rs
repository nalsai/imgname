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
        "rename" => {
            let ext = get_ext(path).unwrap_or_default();
            let datetime = match filetime {
                false => get_datetime(path)?,
                true => get_filedatetime(path)?,
            };

            let newname = date_to_name(&datetime);
            move_file(path, "", &newname, &ext)?;
        }
        "move" => {
            let ext = get_ext(path).unwrap_or_default();
            let datetime = match filetime {
                false => get_datetime(path)?,
                true => get_filedatetime(path)?,
            };

            let filestem = Path::new(path).file_stem().unwrap().to_str().unwrap();
            let subdir = date_to_directory(&datetime);
            move_file(path, &subdir, &filestem, &ext)?;
        }
        "rename-move" => {
            let ext = get_ext(path).unwrap_or_default();
            let datetime = match filetime {
                false => get_datetime(path)?,
                true => get_filedatetime(path)?,
            };

            let subdir = date_to_directory(&datetime);
            let newname = date_to_name(&datetime);
            move_file(path, &subdir, &newname, &ext)?;
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

fn get_ext(path: &str) -> Option<String> {
    let mut ext = Path::new(path)
        .extension()?
        .to_str()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if ext == "jpeg" {
        ext = "jpg".to_string()
    }
    return Some(ext);
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

fn move_file(path: &str, subdir: &str, name: &str, ext: &str) -> Result<(), exif::Error> {
    let dir = Path::new(&path).parent().unwrap();

    let mut new_path = std::path::PathBuf::from(dir);
    new_path.push(&subdir);
    fs::create_dir_all(&new_path).unwrap_or_default();
    new_path.push(&name);
    new_path.set_extension(&ext);

    let mut num = 1;
    while new_path.exists() && !is_same_file(&path, &new_path).unwrap() {
        println!("{} already exists", new_path.to_str().unwrap_or_default());

        let filename = name.to_string() + "-" + &num.to_string();
        num += 1;

        new_path = std::path::PathBuf::from(dir);
        new_path.push(&subdir);
        fs::create_dir_all(&new_path)?;
        new_path.push(&filename);
        new_path.set_extension(&ext);
    }

    println!("{} -> {}", path, new_path.to_str().unwrap_or_default());
    fs::rename(path, new_path)?;
    Ok(())
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
    name += &(dt.hour as u32 * 3600 + dt.minute as u32 * 60 + dt.second as u32).to_string();

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
