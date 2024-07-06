use clap::{arg, Command};

pub fn build_cli() -> Command {
    Command::new("imgname")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("rename")
                .about("Rename the specified file(s)")
                .arg_required_else_help(true)
                .arg(arg!(<PATH> ... "Files to rename").value_parser(clap::value_parser!(String))),
        )
        .subcommand(
            Command::new("move")
                .about("Move the specified file(s) into the subfolder YYYY-MM-DD")
                .arg_required_else_help(true)
                .arg(arg!(<PATH> ... "Files to moves").value_parser(clap::value_parser!(String))),
        )
        .subcommand(
            Command::new("rename-move")
                .about("Rename and moves the specified file(s)")
                .arg_required_else_help(true)
                .arg(arg!(<PATH> ... "Files to rename and move").value_parser(clap::value_parser!(String))),
        )
        .subcommand(
            Command::new("get-date")
                .about("Get the date from the specified filename(s)")
                .arg_required_else_help(true)
                .arg(arg!(<PATH> ... "Files to rename and move").value_parser(clap::value_parser!(String))),
        )
        .subcommand(
            Command::new("get-name")
                .about("Get the name from the specified date(s) in the format: \"2016:05:04 03:02:01\"")
                .arg_required_else_help(true)
                .arg(arg!(<PATH> ... "Files to rename and move").value_parser(clap::value_parser!(String))),
        ).arg(
            clap::arg!(-f --"filetime" "Use last modification time of file instead of exif metadata (can't be combined with -p)")
                .value_parser(clap::value_parser!(bool)),
        ).arg(
            clap::arg!(-p --"pxl" "Use the filename (in the format PXL_20200820_141005222) instead of exif metadata (can't be combined with -f)")
                .value_parser(clap::value_parser!(bool)),
        ).arg(
            clap::arg!(-o --"offset" <HOURS> "Offset in hours to add to the date (use this to set the timezone for videos)")
                .value_parser(clap::value_parser!(i8).range(-23..23)),
        )
}
