use exif::DateTime;
use filetime::FileTime;
use same_file::is_same_file;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::time::{Duration, UNIX_EPOCH};

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut err = true;

    if args.len() > 2 {
        let cmd = args.get(1).unwrap();
        for file in args.split_first().unwrap().1.split_first().unwrap().1 {
            match handle_file(&file, &cmd) {
                Ok(_) => err = false,
                Err(e) => println!("{} {:?}", file, e),
            };
        }
    } else {
        print_help();

        if args.len() == 1 || args.get(1).unwrap() == "help" {
            err = false
        }
    }

    if err {
        std::process::exit(126);
    }
}

fn print_help() {
    println!("Please specify what to do.");
    println!();
    println!("Syntax: imgname [OPTION] [FILES...]");
    println!();
    println!("Options:");
    println!(" rename           renames the specified file(s) using exif");
    println!(" move             moves the specified file(s) into a subfolder using exif");
    println!(" rename-move      renames and moves the specified file(s) using exif");
    println!(" file-rename      renames the specified file(s) using filetime");
    println!(" file-move        moves the specified file(s) using filetime");
    println!(" file-rename-move renames and moves the specified file(s) using filetime");
    println!(" get-date         gets the date from the specified filename(s)");
    println!(" get-name         gets the name from the specified date(s)");
}

fn handle_file(file: &str, cmd: &str) -> Result<(), exif::Error> {
    let ext = Path::new(file)
        .extension()
        .unwrap_or_default()
        .to_ascii_lowercase();

    match cmd {
        "rename" => {
            let datetime = get_datetime(file)?;
            let newname = date_to_name(&datetime);
            move_file(file, "", &newname, ext)?;
        }
        "move" => {
            let datetime = get_datetime(file)?;
            let filestem = Path::new(file).file_stem().unwrap().to_str().unwrap();
            let subdir = date_to_directory(&datetime);
            move_file(file, &subdir, &filestem, ext)?;
        }
        "rename-move" => {
            let datetime = get_datetime(file)?;
            let subdir = date_to_directory(&datetime);
            let newname = date_to_name(&datetime);
            move_file(file, &subdir, &newname, ext)?;
        }
        "file-rename" => {
            let datetime = get_filedatetime(file)?;
            let newname = date_to_name(&datetime);
            move_file(file, "", &newname, ext)?;
        }
        "file-move" => {
            let datetime = get_filedatetime(file)?;
            let filestem = Path::new(file).file_stem().unwrap().to_str().unwrap();
            let subdir = date_to_directory(&datetime);
            move_file(file, &subdir, &filestem, ext)?;
        }
        "file-rename-move" => {
            let datetime = get_filedatetime(file)?;
            let subdir = date_to_directory(&datetime);
            let newname = date_to_name(&datetime);
            move_file(file, &subdir, &newname, ext)?;
        }
        "get-date" => {
            name_to_date_helper(file);
        }
        "test" => {
            let name = Path::new(&file).file_stem().unwrap().to_str().unwrap();
            let dt = name_to_date(&name);
            let new_name = date_to_name(&dt);
            let dt2 = name_to_date(&new_name);
            println!("{} -> {} -> {} -> {}", &name, &dt, &new_name, &dt2);
        }
        _ => {
            print_help();
            std::process::exit(126);
        }
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

fn move_file(path: &str, subdir: &str, name: &str, ext: OsString) -> Result<(), exif::Error> {
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

fn char_to_u8(letter: char) -> u8 {
    return letter.to_ascii_uppercase() as u8 - 64;
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
    let name = Path::new(&path).file_stem().unwrap().to_str().unwrap();
    let clean_name = name.replace("VID_", "");
    let dt = name_to_date(&clean_name);
    println!("{} -> {}", &name, &dt);
}

fn name_to_date(name: &str) -> DateTime {
    let mut chars = name.chars();

    let year = char_to_u8(chars.next().unwrap()) as u16 + 1999;
    let month = char_to_u8(chars.next().unwrap());
    let day_char = chars.next().unwrap();
    let day = if day_char.is_digit(10) {
        day_char.to_digit(10).unwrap() as u8
    } else {
        char_to_u8(day_char) + 9
    };

    let time_string = chars.collect::<String>();
    let time_value = time_string.parse::<u32>().unwrap();

    let hour = time_value / 3600;
    let minute = (time_value - hour * 3600) / 60;
    let second = time_value - hour * 3600 - minute * 60;

    return DateTime {
        year: year,
        month: month,
        day: day,
        hour: hour as u8,
        minute: minute as u8,
        second: second as u8,
        nanosecond: Option::default(),
        offset: Option::default(),
    };
}
