# mediafire_rs

This is a command-line tool written in Rust that allows you to download files or folders from Mediafire URLs.

## Features

- Download files or folders from Mediafire URLs.
- Specify the output directory for downloaded files.
- Supports downloading both individual files and entire folders.

## Prerequisites

Before using this tool, ensure you have the following installed:

- Rust (https://www.rust-lang.org/tools/install)

## Installation

1. Clone this repository:

```
git clone https://github.com/NicKoehler/mediafire_rs.git
```

2. Navigate to the project directory:

```
cd mediafire_rs
```

3. Compile the Rust code:

```
cargo build --release
```

## Usage

Once compiled, you can use the executable binary from the command line as follows:

```
cd target/release
./mdrs "https://www.mediafire.com/file/examplefile12345/file.zip/file"
```

You can also specify the output directory using the `-o` or `--output` flag:

```
./mdrs "https://www.mediafire.com/file/examplefile12345/file.zip/file" -o "./examples"
```

If the output directory is not specified, the files will be downloaded to the current directory.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.