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
            let datetime = match filetime {
                false => get_datetime(path)?,
                true => get_filedatetime(path)?,
            };

            let newname = date_to_name(&datetime);
            move_file(path, "", &newname)?;
        }
        "move" => {
            let datetime = match filetime {
                false => get_datetime(path)?,
                true => get_filedatetime(path)?,
            };

            let filestem = Path::new(path).file_stem().unwrap().to_str().unwrap();
            let subdir = date_to_directory(&datetime);
            move_file(path, &subdir, &filestem)?;
        }
        "rename-move" => {
            let datetime = match filetime {
                false => get_datetime(path)?,
                true => get_filedatetime(path)?,
            };

            let subdir = date_to_directory(&datetime);
            let newname = date_to_name(&datetime);
            move_file(path, &subdir, &newname)?;
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
fn files_with_same_extension_at_different_paths_exist_and_are_equal(path1: &Path, path2: &Path) -> bool {
    let path1_ext = path1.extension().unwrap().to_str().unwrap();
    let path2_ext = path2.extension().unwrap().to_str().unwrap();

    if exists_and_is_same_file(&path1, &path2.with_extension(&path1_ext)) {
        
        if exists_and_is_same_file(&path2, &path1.with_extension(&path2_ext)) {
            return true;
        }

    }

    false
}

fn exists_and_is_same_file(path1: &Path, path2: &Path) -> bool {
    if path1.exists() && path2.exists() {
        return is_same_file(path1, path2).unwrap();
    }
    return false;
}


fn move_file(src_path: &str, dest_subdir: &str, original_dest_name_stem: &str) -> Result<(), exif::Error> {
    // TODO: handle errors
    let src_filepath = Path::new(&src_path);
    //let src_dir = src_filepath.parent().unwrap();
    let src_parent = src_filepath.parent().unwrap();
    let src_dir = if src_parent.as_os_str().is_empty() { Path::new(".") } else { src_parent };
    
    let src_name_stem = src_filepath.file_stem().unwrap();
    
    let dest_dir = &src_dir.join(&dest_subdir);
    let mut dest_name_stem: String = original_dest_name_stem.to_string();

    // make sure the dest_name_stem is unique
    let mut counter = 1;
    for entry in fs::read_dir(src_dir)? {
        match entry {
            Ok(entry) => {
                let entry_path = entry.path();
                if entry_path.file_stem().unwrap().to_str().unwrap() == dest_name_stem {
    
                    if !files_with_same_extension_at_different_paths_exist_and_are_equal(&src_filepath, &entry_path) {
                        println!("{} already exists", entry_path.to_str().unwrap_or_default());
                        dest_name_stem = format!("{}-{}", original_dest_name_stem, counter);
                        counter += 1;
                    }
                }
            },
            Err(_) => {
                println!("Error reading entry");
                continue
            },
        }
    }

    // find and move all files with src_name_stem in src_dir to dest_dir with dest_name_stem
    for entry in fs::read_dir(src_dir)? {
        match entry {
            Ok(entry) => {

                // If the file name matches src_name, move it to dest_dir with dest_name
                let entry_path = entry.path();
                if entry_path.file_stem().unwrap() == src_name_stem {
                    let dest_ext = entry_path.extension().unwrap().to_str().unwrap().to_ascii_lowercase();

                    let dest_path = dest_dir.join(&dest_name_stem).with_extension(&dest_ext);

                    println!("{} -> {}", entry_path.to_str().unwrap_or_default(), dest_path.to_str().unwrap_or_default());
                    
                    fs::rename(entry.path(), dest_path)?;
                }
            },
            Err(_) => {
                println!("Error reading entry");
                continue
            },
        }
    }
    


    // TODO: handle filename collisions

        /*
    
    let mut new_path = std::path::PathBuf::from(src_dir);
    new_path.push(&dest_subdir);
    fs::create_dir_all(&new_path).unwrap_or_default();
    new_path.push(&dest_name);
    new_path.set_extension(&src_ext);

    let mut num = 1;
    while new_path.exists() && !is_same_file(&src_path, &new_path).unwrap() {
        println!("{} already exists", new_path.to_str().unwrap_or_default());

        let filename = dest_name.to_string() + "-" + &num.to_string();
        num += 1;

        new_path = std::path::PathBuf::from(src_dir);
        new_path.push(&dest_subdir);
        fs::create_dir_all(&new_path)?;
        new_path.push(&filename);
        new_path.set_extension(&src_ext);
    }

    println!("{} -> {}", path, new_path.to_str().unwrap_or_default());

    fs::rename(src_path, new_path)?;


    //move every file in src_dir with dest_name to src_path 

    //println!("{}, {:?}, {}, {}, {}", path, dir, subdir, name, ext);

    //for file in files {
        
    //}
    
     */
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
