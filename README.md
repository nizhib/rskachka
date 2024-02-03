# rskachka

Download images _**blazingly fast ©**_.

## Installation

```bash
cargo install --git https://github.com/nizhib/rskachka
```

## Usage

```text
Usage: rskachka [OPTIONS] -i <INDEX_PATH> -o <OUTPUT_ROOT>

Options:
  -i, --index-path <INDEX_PATH>      Index file path
  -o, --output-root <OUTPUT_ROOT>    Output images root
  -f, --fields <FIELDS>              ID fields [default: 0]
  -u, --url-field <URL_FIELD>        URL field [default: -1]
  -t, --timeout <TIMEOUT>            Timeout for requests, in seconds [default: 5]
  -m, --max-size <MAX_SIZE>          Output images max size [default: 640]
  -j, --jpeg-quality <JPEG_QUALITY>  Output images jpeg quality [default: 90]
  -w, --worker-count <WORKER_COUNT>  Concurrent workers count [default: 32]
  -r, --resume                       Resume last run if any
  -v, --verbose...                   Increase logging verbosity
  -q, --quiet...                     Decrease logging verbosity
  -p, --progress                     Show progressbar
  -n, --no-header                    No header in index
  -h, --help                         Print help
```
