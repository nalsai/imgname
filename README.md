# Imgname

A command line app similar to [ImgReName](https://github.com/Nalsai/ImgReName/), but written in Rust 🦀, with the added ability to convert the name back to the date.

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

## License

Imgname is distributed under the terms of both the MIT License and the Apache License (Version 2.0).

See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.
