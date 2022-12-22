# Imgname

A command line app similar to [ImgReName](https://github.com/Nalsai/ImgReName/), but written in Rust 🦀, with the added ability to convert the name back to the date.

```
Syntax: imgname [OPTION] [FILES...]

Options:
 rename           renames the specified file(s) using exif
 move             moves the specified file(s) into a subfolder using exif
 rename-move      renames and moves the specified file(s) using exif
 file-rename      renames the specified file(s) using filetime
 file-move        moves the specified file(s) using filetime
 file-rename-move renames and moves the specified file(s) using filetime
 get-date         gets the date from the specified filename(s)
 get-name         gets the name from the specified date(s)
```

## License

Imgname is distributed under the terms of both the MIT License and the Apache License (Version 2.0).

See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.
