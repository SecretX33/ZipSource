# ZipSource
A way to quickly zip source code in a folder, written in Rust. It was made for the cases where you need to zip the contents of a folder based on your own `.gitignore` files.

## Use-case
The inspiration for this project is that I worked with a client that did not use GitHub, but still demanded the source code of the private project. Without this tool, I had to manually zip the entire folder, then delete the binary files from it, every, single, time. With this tool, I just run the command and let it do its thing.

## Instructions

You can provide the base path to be used, or if none is provided then ZipSource will use the current directory instead. You can also provide the file name if you'd like to modify it.

### Command usage

Zips the current directory into a file that follows the following naming scheme: `{folder_name} (Source Code).zip`. The `.zip` file will be generated and placed inside the folder.
```shell
zipsource
```

Zips the contents of `C:\Projects\myproject` into a file that follows the following naming scheme: `{folder_name} (Source Code).zip`.
```shell
zipsource "C:\Projects\myproject"
```

Zips the contents of `C:\Projects\myproject` into a file called `any name you would like.zip`.
```shell
zipsource "C:\Projects\myproject" "any name you would like.zip"
```

## Compile

- Install [Rust](https://www.rust-lang.org/tools/install).
- Open the source code folder and run `cargo build` for a develop build, or `cargo build --release` for a release (optimized) build.

## License

This project is licensed under [MIT License](LICENSE).