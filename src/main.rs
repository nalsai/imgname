use exif::DateTime;
use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "rename" {
        for file in args.split_first().unwrap().1.split_first().unwrap().1 {
            rename_and_move(file);
        }
    } else if args.len() > 1 && args[1] == "get-date" {
        for file in args.split_first().unwrap().1.split_first().unwrap().1 {
            get_date_of_file(file);
        }
    } else if args.len() > 1 && args[1] == "test" {
        for file in args.split_first().unwrap().1.split_first().unwrap().1 {
            let name = Path::new(&file).file_stem().unwrap().to_str().unwrap();

            let dt = name_to_date(&name);
            let new_name = date_to_name(&dt);
            let dt2 = name_to_date(&name);
            println!("{} -> {} -> {} -> {}", &name, &dt, &new_name, &dt2);
        }
    } else {
        println!("Please specify what to do.");
        println!();
        println!("Syntax: imgname [OPTION] [FILES...]");
        println!();
        println!("Options:");
        println!(" rename     renames the specified file(s)");
        println!(" get-date   gets the date from the specified filename(s)");
        println!(" test       gets the date from the specified filename(s), gets the name for that date and gets the date from that name again");
    }
}

fn rename_and_move(path: &str) {
    let file = std::fs::File::open(&path).unwrap();
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();
    let exif = exifreader.read_from_container(&mut bufreader).unwrap();

    let name = Path::new(&path).file_stem().unwrap().to_str().unwrap();
    if let Some(field) = exif.get_field(exif::Tag::DateTime, exif::In::PRIMARY) {
        match field.value {
            exif::Value::Ascii(ref vec) if !vec.is_empty() => {
                if let Ok(dt) = DateTime::from_ascii(&vec[0]) {
                    let new_name = date_to_name(&dt);
                    // TODO: extension, folder name, move
                    println!("{}: {} -> {}", &name, &dt, &new_name);
                }
            }
            _ => {}
        }
    }
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
    return letter as u8 - 64;
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
