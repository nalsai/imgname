# Imgname

A command line app similar to [ImgReName](https://github.com/Nalsai/ImgReName/), but written in Rust 🦀, with the added ability to convert the name back to the date.

It names the files by making a letter out of the year and the month (2000=A ,2001=B, ...) (January=A, February=B, ...).
If the day of the month is less than or equal to 9, it directly uses the number, otherwise it also turns it into a letter (10=A, 11=B, ...).
The time value is calculated using `hour * 3600 + minute * 60 + second`.

This results in a name like AA100000 for 2000-01-01 00:00:00 and ZLV86399 for 2025-12-31 23:59:59.
These values are the minimum and maximum date supported.

It also recognizes files with the same name and a different extension as the same file, and it will move/rename them too. This is useful when you have a RAW file and a JPEG file with the same name, as only TIFF based RAW images (like Sony ARW) are supported directly.

```
Usage: imgname [OPTIONS] <COMMAND>

Commands:
  rename       Rename the specified file(s)
  move         Move the specified file(s) into the subfolder YYYY-MM-DD
  rename-move  Rename and moves the specified file(s)
  get-date     Get the date from the specified filename(s)
  get-name     Get the name from the specified date(s) (format: "2016:05:04 03:02:01")
  help         Print this message or the help of the given subcommand(s)

Options:
  -f, --filetime  Use last modification time of file instead of exif metadata
  -h, --help      Print help information
```

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

## TODO

- Add/improve error handling
- Check for extensions with multiple dots (e.g. `.out.pp3`)
- Write tests
- Rename by incrementing the seconds for the name if the file already exists instead of adding a number at the end

## License

Imgname is distributed under the terms of both the MIT License and the Apache License (Version 2.0).

See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.
