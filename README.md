# 🦀🏋️ rskachka

Download images **blazingly fast**.

## 🏗️ Installation

```bash
cargo install --git https://github.com/nizhib/rskachka
```

## 🚀 Usage

### 💾 Download images

```text
Usage: rskachka [OPTIONS] -i <SOURCE_PATH> -o <OUTPUT_ROOT>

Options:
  -s, --source-path <SOURCE_PATH>    Source file location
  -o, --output-root <OUTPUT_ROOT>    Output images root
  -f, --fields <FIELDS>              ID fields indexes [default: 0]
  -u, --url-field <URL_FIELD>        URL field index [default: -1]
  -t, --timeout <TIMEOUT>            Timeout for requests, in seconds [default: 5]
  -m, --max-size <MAX_SIZE>          Output images max size [default: 640]
  -j, --jpeg-quality <JPEG_QUALITY>  Output images jpeg quality [default: 90]
  -w, --worker-count <WORKER_COUNT>  Concurrent workers count [default: 32]
  -r, --resume                       Resume last run if any
  -v, --verbose...                   Increase logging verbosity
  -q, --quiet...                     Decrease logging verbosity
  -p, --progress                     Show progressbar
  -n, --no-header                    Use the first line in source
  -h, --help                         Print help
```

### 👷🕵️ Build an index and check missing images

```text
Usage: rsindex [OPTIONS] -s <SOURCE_PATH> -i <INDEX_PATH> -o <OUTPUT_ROOT> [-m <MISSING_PATH>]

Options:
  -s, --source-path <SOURCE_PATH>    Source file location
  -i, --index-path <INDEX_PATH>      Index file location
  -m, --missing-path <MISSING_PATH>  Missing file location
  -o, --output-root <OUTPUT_ROOT>    Images output root
  -u, --url-field <URL_FIELD>        URL field index [default: -1]
  -n, --no-header                    Use the first line in source
  -p, --progress                     Show progressbar
  -h, --help                         Print help
```
