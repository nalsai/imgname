use exif::DateTime;
use filetime::FileTime;
use std::env;
use std::fs;
use std::path::Path;
use std::time::{Duration, UNIX_EPOCH};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 && args[1] == "rename" {
        for file in args.split_first().unwrap().1.split_first().unwrap().1 {
            rename_and_move(file);
        }
    } else if args.len() > 2 && args[1] == "rename-filetime" {
        for file in args.split_first().unwrap().1.split_first().unwrap().1 {
            rename_and_move_filetime(file);
        }
    } else if args.len() > 2 && args[1] == "get-date" {
        for file in args.split_first().unwrap().1.split_first().unwrap().1 {
            get_date_of_file(file);
        }
    } else if args.len() > 2 && args[1] == "test" {
        for file in args.split_first().unwrap().1.split_first().unwrap().1 {
            let name = Path::new(&file).file_stem().unwrap().to_str().unwrap();

            let dt = name_to_date(&name);
            let new_name = date_to_name(&dt);
            let dt2 = name_to_date(&name);
            println!("{} -> {} -> {} -> {}", &name, &dt, &new_name, &dt2);
        }
    } else if args.len() <= 1 || (args.len() > 1 && args[1] == "help") {
        print_help();
    } else {
        print_help();
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
    println!(" rename-filetime  renames the specified file(s) using filetime");
    println!(" get-date         gets the date from the specified filename(s)");
    println!(" test             gets the date from the specified filename(s), gets the name for that date and gets the date from that name again");
}

fn rename_and_move(path: &str) {
    let file = std::fs::File::open(&path).unwrap();

    let dir = Path::new(&path).parent().unwrap();
    let ext = Path::new(&path).extension().unwrap().to_ascii_lowercase();
    let name_ext_str = Path::new(&path).file_name().unwrap().to_str().unwrap();

    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();

    match exifreader.read_from_container(&mut bufreader) {
        Ok(exif) => match exif.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY) {
            Some(field) => match field.value {
                exif::Value::Ascii(ref vec) if !vec.is_empty() => {
                    if let Ok(mut dt) = DateTime::from_ascii(&vec[0]) {
                        let mut new_name = date_to_name(&dt);

                        let mut new_path = std::path::PathBuf::from(dir);
                        let new_folder_name =
                            format!("{:04}-{:02}-{:02}", dt.year, dt.month, dt.day);
                        let new_folder = Path::new(&new_folder_name);
                        new_path.push(&new_folder);
                        fs::create_dir_all(&new_path).unwrap();

                        new_path.push(&new_name);
                        new_path.set_extension(&ext);

                        while new_path.exists() {
                            println!(
                                "{}.{} already exists, incrementing by 1",
                                &new_name,
                                &ext.to_str().unwrap()
                            );

                            dt = DateTime {
                                year: dt.year,
                                month: dt.month,
                                day: dt.day,
                                hour: dt.hour,
                                minute: dt.minute,
                                second: dt.second + 1,
                                nanosecond: dt.nanosecond,
                                offset: dt.offset,
                            };
                            new_path = [dir, new_folder].iter().collect();
                            new_name = date_to_name(&dt);
                            new_path.push(&new_name);
                            new_path.set_extension(&ext);
                        }
                        fs::rename(path, new_path).unwrap();

                        println!("{}: {} -> {}", &name_ext_str, &dt, &new_name);
                    } else {
                        println!(
                            "{} has unreadable date time in exif metadata, skipped",
                            &name_ext_str
                        )
                    }
                }
                _ => {
                    println!(
                        "{} has unreadable date time in exif metadata, skipped",
                        &name_ext_str
                    )
                }
            },
            None => {
                println!(
                    "{} doesn't have date time in exif metadata, skipped",
                    &name_ext_str
                )
            }
        },
        Err(_e) => println!("{} has no exif metadata, skipped", &name_ext_str),
    }
}

fn rename_and_move_filetime(path: &str) {
    let dir = Path::new(&path).parent().unwrap();
    let ext = Path::new(&path).extension().unwrap().to_ascii_lowercase();
    let name_ext_str = Path::new(&path).file_name().unwrap().to_str().unwrap();

    let metadata = fs::metadata(path).unwrap();

    let mtime = FileTime::from_last_modification_time(&metadata);
    let datetime = chrono::DateTime::<chrono::Local>::from(
        UNIX_EPOCH + Duration::from_secs(mtime.unix_seconds() as u64),
    );

    let timestamp_str = datetime.format("%Y:%m:%d %H:%M:%S").to_string();
    let mut dt = DateTime::from_ascii(timestamp_str.as_bytes()).unwrap();

    let mut new_name = date_to_name(&dt);

    let mut new_path = std::path::PathBuf::from(dir);
    let new_folder_name = format!("{:04}-{:02}-{:02}", dt.year, dt.month, dt.day);
    let new_folder = Path::new(&new_folder_name);
    new_path.push(&new_folder);
    fs::create_dir_all(&new_path).unwrap();

    new_path.push(&new_name);
    new_path.set_extension(&ext);

    while new_path.exists() {
        println!(
            "{}.{} already exists, incrementing by 1",
            &new_name,
            &ext.to_str().unwrap()
        );

        dt = DateTime {
            year: dt.year,
            month: dt.month,
            day: dt.day,
            hour: dt.hour,
            minute: dt.minute,
            second: dt.second + 1,
            nanosecond: dt.nanosecond,
            offset: dt.offset,
        };
        new_path = [dir, new_folder].iter().collect();
        new_name = date_to_name(&dt);
        new_path.push(&new_name);
        new_path.set_extension(&ext);
    }
    fs::rename(path, new_path).unwrap();

    println!("{}: {} -> {}", &name_ext_str, &dt, &new_name);
}

fn get_date_of_file(path: &str) {
    let name = Path::new(&path).file_stem().unwrap().to_str().unwrap();
    let clean_name = name.replace("VID_", "");
    let dt = name_to_date(&clean_name);
    println!("{} -> {}", &name, &dt);
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
