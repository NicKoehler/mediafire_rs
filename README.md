<p align="center" >
  <img src="assets/logo.svg" alt="logo">
</p>

## Preview

<img src="assets/demo.gif" alt="mediafire_rs">

async rust rewrite of [mediafire_bulk_downloader](https://github.com/nickoehler/mediafire_bulk_downloader)

## Installation

```bash
cargo install mediafire_rs
```

## Usage

```bash
Usage: mdrs.exe [OPTIONS] <URL>

Arguments:
  <URL>  Folder or file to download

Options:
  -o, --output <OUTPUT>  Output directory [default: .]
  -m, --max <MAX>        Maximum number of concurrent downloads [default: 10]
  -h, --help             Print help
  -V, --version          Print version
```

For building from source, please refer to the [BUILDING.md](BUILDING.md) file for detailed instructions.
