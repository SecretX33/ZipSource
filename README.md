# ZipSource
A way to quickly zip source code in a folder, written in Rust. It was made for the cases where you need to zip a project folder according to its `.gitignore` file. 

The best part is: it's very easy to use, and really fast.

## Download

ZipSource supports Windows and Linux (but technically works and can be compiled to MacOS too).

[Click here](https://github.com/SecretX33/ZipSource/releases/latest) to download the precompiled executable for your OS.

## Use-case
The inspiration for this project is that I worked with a client that didn't use GitHub, but still demanded the source code of the private project. Without this tool, I had to manually zip the entire folder, then delete the binary files from it, every, single, time. With this tool, I just run the command and let it do its thing.

And because it is a CLI application, you can use it in conjunction with any other tool that is compatible. Going back to the previous example, I wrote a small `Exec` task in Gradle (Java's build tool) to run ZipSource on the project's folder (example code can be seen [here](examples/gradle)).

## Instructions

You can provide the base path to be used, or if none is provided then ZipSource will use the current directory instead. You can also provide a file name for the zip if you'd like to modify it. The `zip` file will be generated and placed inside the project folder (base path).

ZipSource usage is consistent across different OSes.

### Command usage

Zips the current directory into a file that follows the following naming scheme: `{folder_name} (Source Code).zip`.
```shell
zipsource
```

Zips the contents of `C:\Projects\myproject` into a file named `myproject (Source Code).zip`.
```shell
zipsource "C:\Projects\myproject"
```

Zips the contents of `C:\Projects\myproject` into a file named `any name you would like.zip`.
```shell
zipsource "C:\Projects\myproject" "any name you would like.zip"
```

## Compile from source

- Install [Rust](https://www.rust-lang.org/tools/install).
- Open the source code folder and run `cargo build` for a develop build, or `cargo build --release` for a release (optimized) build.

## License

This project is licensed under [MIT License](LICENSE).
