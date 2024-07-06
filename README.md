# Imgname

A command line app similar to [ImgReName](https://github.com/Nalsai/ImgReName/), but written in Rust 🦀, with the ability to convert the name back to the date.

It names the files by making a letter out of the year and the month (2000=A ,2001=B, ...) (January=A, February=B, ...).
If the day of the month is less than or equal to 9, it directly uses the number, otherwise it also turns it into a letter (10=A, 11=B, ...).
The time value is calculated using `hour * 3600 + minute * 60 + second`.

This results in a name like AA100000 for 2000-01-01 00:00:00 and ZLV86399 for 2025-12-31 23:59:59.
These values are the minimum and maximum date currently supported. Other dates will have non alphanumeric characters in their name.

It also recognizes multiple files with the same name but a different extension, and will move/rename them together. This is useful when you have a JPEG file and a RAW file with the same name.

<pre><u style="text-decoration-style:solid"><b>Usage:</b></u> <b>imgname</b> [OPTIONS] &lt;COMMAND&gt;

<u style="text-decoration-style:solid"><b>Commands:</b></u>
  <b>rename</b>       Rename the specified file(s)
  <b>move</b>         Move the specified file(s) into the subfolder YYYY-MM-DD
  <b>rename-move</b>  Rename and moves the specified file(s)
  <b>get-date</b>     Get the date from the specified filename(s)
  <b>get-name</b>     Get the name from the specified date(s) in the format: &quot;2016:05:04 03:02:01&quot;
  <b>help</b>         Print this message or the help of the given subcommand(s)

<u style="text-decoration-style:solid"><b>Options:</b></u>
  <b>-f</b>, <b>--filetime</b>        Use last modification time of file instead of exif metadata (can&apos;t be combined with -p)
  <b>-p</b>, <b>--pxl</b>             Use the filename (in the format PXL_20200820_141005222) instead of exif metadata (can&apos;t be combined with -f)
  <b>-o</b>, <b>--offset</b> &lt;HOURS&gt;  Offset in hours to add to the date (use this to set the timezone for videos)
  <b>-h</b>, <b>--help</b>            Print help</pre>

## Installing

```sh
git clone git@github.com:Nalsai/imgname.git
cd imgname
cargo install --path .

# Install shell completion (replace HASH with the output at compile time, path may vary depending on your system)
sudo install -m644 ./target/release/build/imgname-HASH/out/imgname.bash /usr/share/bash-completion/completions/imgname  # Bash
sudo install -m644 ./target/release/build/imgname-HASH/out/imgname.fish /usr/share/fish/completions/imgname.fish        # Fish
sudo install -m644 ./target/release/build/imgname-HASH/out/_imgname /usr/share/zsh/site-functions/_imgname              # Zsh
```

## License

Imgname is distributed under the terms of both the MIT License and the Apache License (Version 2.0).

See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.
