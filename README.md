
<p align="center" >
  <img src="assets/logo.svg" alt="logo">
  <strong>
    Async rust rewrite of
    <a href="https://github.com/nickoehler/mediafire_bulk_downloader">
      mediafire_bulk_downloader
    </a>
  </strong>
</p>

## Preview

<img src="assets/demo.gif" alt="mediafire_rs">


## Installation

```bash
cargo install mediafire_rs
```

## Usage

```bash
Usage: mdrs [OPTIONS] <URLS>...

Arguments:
  <URLS>...  List of folders or files to download

Options:
  -i, --input <FILE>     File containing URLs
  -o, --output <OUTPUT>  Output directory [default: .]
  -m, --max <MAX>        Maximum number of concurrent downloads [default: 10]
  -t, --tries <MAX>      Maximum number of tries to repeat for every download [default: 1]
  -r, --reverse          Download files in reverse order (largest first)
  -p, --proxy <FILE>     Specify a file to read proxies from
      --proxy-download   Downloads files through proxies, the default is to use proxies for the API only
  -h, --help             Print help
  -V, --version          Print version
```
## Building
For building from source, please refer to the [BUILDING.md](BUILDING.md) file for detailed instructions.

## Contributing
For contributions, please refer to the [CONTRIBUTING](CONTRIBUTING.md) file for detailed instructions.
