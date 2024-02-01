# rskachka

Download images _**blazingly fast ©**_.

## Installation

```bash
cargo install --git https://github.com/nizhib/rskachka
```

## Usage

```text
Usage: rskachka [OPTIONS] --file-path <FILE_PATH> --output-root <OUTPUT_ROOT>

Options:
  -f, --file-path <FILE_PATH>        Index file path
  -o, --output-root <OUTPUT_ROOT>    Images output root
  -i, --id-fields <ID_FIELDS>        ID fields [default: 0]
  -u, --url-field <URL_FIELD>        URL field [default: -1]
  -j, --jpeg-quality <JPEG_QUALITY>  Output images quality [default: 90]
  -m, --max-size <MAX_SIZE>          Output image size limit [default: 640]
  -w, --worker-count <WORKER_COUNT>  Concurrent workers [default: 32]
  -r, --resume                       Resume last run if any
  -v, --verbose...                   Increase logging verbosity
  -q, --quiet...                     Decrease logging verbosity
  -p, --progress                     Show progressbar
  -n, --no-header                    CSV file has no header
  -h, --help                         Print help
```
