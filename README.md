# ZipSource
A way to quickly zip source code in a folder, written in Rust. It was made for the cases where you need to zip the contents of a folder based on your own `.gitignore` file.

## Use-case
The inspiration for this project is that I worked with a client that did not use GitHub, but still demanded the source code of the private project. Without this tool, I had to manually zip the entire folder, then delete the binary files from it, every, single, time. With this tool, I just run the command and let it do its thing. 

## Instructions

You can provide the base path to be used, or if none is provided then ZipSource will use the current directory instead.

### Command usage

Zips the current directory.
```shell
zipsource
```

Zips the contents of `C:\Projects\myproject`.
```shell
zipsource "C:\Projects\myproject"
```