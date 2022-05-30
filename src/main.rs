use exif::DateTime;

fn main() {
    let dt = DateTime::from_ascii(b"2020:05:30 17:40:59").unwrap();
    let name = date_to_name(&dt);
    let dt2 = name_to_date(&name);
    println!("{}, {} -> {}", &name, &dt.to_string(), &dt2.to_string());
}

fn u8_to_string(num: u8, uppercase: bool) -> String {
    return ((num + if uppercase { 64 } else { 96 }) as char).to_string();
}

fn string_to_u8(letter: String) -> u8 {
    return letter.to_uppercase().chars().next().unwrap() as u8 - 64;
}

fn char_to_u8(letter: char) -> u8 {
    return string_to_u8(letter.to_string());
}

fn date_to_name(dt: &DateTime) -> String {
    let mut name = String::default();
    name += &u8_to_string((dt.year - 1999) as u8, true);
    name += &u8_to_string(dt.month, true);
    if dt.day <= 9 {
        name += &dt.day.to_string();
    } else {
        name += &u8_to_string(dt.day - 9, true);
    }
    name += &(dt.hour as u32 * 3600 + dt.minute as u32 * 60 + dt.second as u32).to_string();

    return name;
}

fn name_to_date(name: &String) -> DateTime {
    let mut chars = name.chars();

    let year = string_to_u8(chars.next().unwrap().to_string()) as u16 + 1999;
    let month = string_to_u8(chars.next().unwrap().to_string());
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
