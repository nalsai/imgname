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

fn move_file(method: &str, src_path_str: &str, mut datetime: DateTime) -> Result<(), exif::Error> {
    let src_path = Path::new(&src_path_str);
    let src_dir = if src_path.parent().unwrap().as_os_str().is_empty() {
        Path::new(".")
    } else {
        src_path.parent().unwrap()
    };
    let src_name_stem = src_path.file_stem().unwrap();

    let dest_subdir = if method == "move" || method == "rename-move" {
        date_to_directory(&datetime)
    } else {
        String::default()
    };
    let dest_name = if method == "rename" || method == "rename-move" {
        date_to_name(&datetime)
    } else {
        Path::new(src_path_str)
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string()
    };
    let dest_dir = &src_dir.join(&dest_subdir);

    // increment name until it is unique in both src_dir and dest_dir
    let mut dest_name_unique = dest_name.clone();
    let mut counter = 1;
    while another_file_with_stem_exists(&src_dir, &dest_name_unique, src_path)
        || (!dest_subdir.is_empty()
            && another_file_with_stem_exists(&dest_dir, &dest_name_unique, src_path))
    {
        println!("{} already exists", dest_name_unique);
        if method == "rename" || method == "rename-move" {
            datetime.second += 1;
            dest_name_unique = date_to_name(&datetime);
        } else {
            dest_name_unique = format!("{}-{}", dest_name, counter);
            counter += 1;
        }
    }

    // find all files with src_name_stem in src_dir
    let iter = fs::read_dir(src_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().unwrap().is_file())
        .filter(|e| {
            e.file_name()
                .to_str()
                .unwrap()
                .starts_with(src_name_stem.to_str().unwrap())
        });

    fs::create_dir(&dest_dir).unwrap_or_default();

    // move files to dest_dir
    for entry in iter {
        let dest_ext = entry
            .path()
            .extension()
            .unwrap()
            .to_str()
            .unwrap()
            .to_ascii_lowercase(); // TODO: jpeg -> jpg
        let dest_path = dest_dir.join(&dest_name_unique).with_extension(&dest_ext);

        println!(
            "{} -> {}",
            entry.path().to_str().unwrap_or_default(),
            dest_path.to_str().unwrap_or_default()
        );

        // make absolutely sure no file will be overwritten - this should never happen
        if dest_path.exists() && !is_same_file(entry.path(), &dest_path).unwrap() {
            panic!(
                "Error: trying to move {} to {} but a file already exists at the target",
                entry.path().to_str().unwrap_or_default(),
                dest_path.to_str().unwrap_or_default()
            );
        }
        fs::rename(entry.path(), dest_path)?;
    }

    Ok(())
}

fn another_file_with_stem_exists(path: &Path, file_stem: &str, src_path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    // find all files with the same name (stem) in the directory
    let iter = fs::read_dir(path)
        .expect("Failed to read directory")
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().expect("Failed to get file type").is_file())
        .filter(|e| e.file_name().to_str().unwrap().starts_with(file_stem));

    // compare the file with the same name and extension
    for entry in iter {
        if !files_with_same_extension_are_equal(src_path, &entry.path()) {
            return true;
        }
    }

    false
}

/* Compare if a file path1.a is equal to path2.a and path1.b to path2.b */
fn files_with_same_extension_are_equal(path1: &Path, path2: &Path) -> bool {
    if !path1.exists() || !path2.exists() {
        return false;
    }

    let ext1 = path1.extension().unwrap().to_str().unwrap();
    let ext2 = path2.extension().unwrap().to_str().unwrap();

    let path1_with_ext2 = path1.with_extension(&ext2);
    let path2_with_ext1 = path2.with_extension(&ext1);

    if !path1_with_ext2.exists() || !path2_with_ext1.exists() {
        return false;
    }

    is_same_file(&path1, &path2_with_ext1).unwrap()
        && is_same_file(&path2, path1_with_ext2).unwrap()
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
