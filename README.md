# primg

[Primitive Pictures](https://github.com/fogleman/primitive) ported to Rust.

Some features missing, but roughly 1.5x faster.

## Usage

```
$ cargo run --bin main --release -- --help
    Finished release [optimized] target(s) in 0.0 secs
     Running `target/release/main --help`
primg

USAGE:
    main [OPTIONS] <INFILE> <OUTFILE>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --alpha <alpha>                Alpha (1-255) [default: 128]
    -n, --num-shapes <num-shapes>      Number of shapes [default: 100]
    -s, --output-size <output-size>    Output size [default: 1024]
    -q, --quality <quality>            Quality (1-3) [default: 2]
    -t, --shape <shape>                Shape type (triangle, ellipse, rectangle, rotated-rectangle) [default: triangle]

ARGS:
    <INFILE>     Path to image file
    <OUTFILE>    Output file path
```
