extern crate clap;
extern crate primg;

use clap::{Arg, App};

fn main() {
    let matches = App::new("primg")
        .arg(Arg::with_name("shape")
            .help("Shape type")
            .short("t")
            .long("shape")
            .takes_value(true)
            .default_value("triangle"))
        .arg(Arg::with_name("num-shapes")
            .help("Number of shapes")
            .short("n")
            .long("num-shapes")
            .takes_value(true)
            .default_value("100"))
        .arg(Arg::with_name("output-size")
            .help("Output size")
            .short("s")
            .long("output-size")
            .takes_value(true)
            .default_value("1024"))
        .arg(Arg::with_name("alpha")
            .help("Alpha (1-255)")
            .short("a")
            .long("alpha")
            .takes_value(true)
            .default_value("128"))
        .arg(Arg::with_name("quality")
            .help("Quality (1-4)")
            .short("q")
            .long("quality")
            .takes_value(true)
            .default_value("2"))
        .arg(Arg::with_name("INFILE")
            .help("Path to image file")
            .required(true))
        .arg(Arg::with_name("OUTFILE")
            .help("Output file path")
            .required(true))
        .get_matches();

    let in_path = String::from(matches.value_of("INFILE").unwrap());
    let out_path = String::from(matches.value_of("OUTFILE").unwrap());
    let num_shapes = matches.value_of("num-shapes").unwrap().parse::<u32>().unwrap();
    let shape_type = match matches.value_of("shape").unwrap().to_lowercase().as_ref() {
        "triangle" => primg::ShapeType::Triangle,
        _ => panic!("invalid shape"),
    };
    let out_size = matches.value_of("output-size").unwrap().parse::<usize>().unwrap();
    let alpha = matches.value_of("alpha").unwrap().parse::<u8>().unwrap();
    let per_worker = matches.value_of("quality").unwrap().parse::<u8>().unwrap();

    assert!(alpha > 0, "alpha must be between 1-255");
    assert!(1 <= per_worker && per_worker <= 4, "quality must be between 1-4");

    let config = primg::Config {
        in_path,
        out_path,
        num_shapes,
        shape_type,
        out_size,
        alpha,
        per_worker,
    };
    primg::run(config);
}
