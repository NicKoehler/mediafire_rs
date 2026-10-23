
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
Usage: mdrs [OPTIONS] <URLS|--input <FILE>>

Arguments:
  [URLS]...  List of folders or files to download

Options:
  -i, --input <FILE>        File containing URLs
  -o, --output <OUTPUT>     Output directory [default: .]
  -m, --max <MAX>           Maximum concurrent downloads [default: 10]
  -t, --tries <TRIES>       Maximum retries per download [default: 1]
  -r, --reverse             Download largest files first
  -p, --proxy <FILE>        File containing proxy list
      --proxy-download      Use proxies for file downloads
      --flare-solver <URL>  Base URL of a self-hosted FlareSolverr instance (e.g. http://127.0.0.1:8191) used for Cloudflare-challenged downloads
  -h, --help                Print help
  -V, --version             Print version
```

Some MediaFire pages are protected by an interactive Cloudflare challenge that a regular HTTP client cannot get past. If your downloads fail with a `403` on such pages, run a [FlareSolverr](https://github.com/FlareSolverr/FlareSolverr) instance locally (for example `docker run -d -p 8191:8191 ghcr.io/flaresolverr/flaresolverr`) and pass its URL with `--flare-solver`. Only challenged files are sent to FlareSolverr; everything else downloads as usual.
## Building
For building from source, please refer to the [BUILDING.md](BUILDING.md) file for detailed instructions.

## Contributing
For contributions, please refer to the [CONTRIBUTING](CONTRIBUTING.md) file for detailed instructions.
