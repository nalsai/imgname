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
                .about("Get the name from the specified date(s) (format: \"2016:05:04 03:02:01\")")
                .arg_required_else_help(true)
                .arg(arg!(<PATH> ... "Files to rename and move").value_parser(clap::value_parser!(String))),
        ).arg(
            clap::arg!(-f --"filetime" "Use last modification time of file instead of exif metadata")
                .value_parser(clap::value_parser!(bool)),)
}
