extern crate clap;
extern crate primg;

use clap::{Arg, App};

fn main() {
    let matches = App::new("primg")
        .arg(Arg::with_name("shape")
            .help("Shape type")
            .short("s")
            .long("shape")
            .takes_value(true)
            .default_value("triangle"))
        .arg(Arg::with_name("n")
            .help("Number of shapes")
            .short("n")
            .long("n")
            .takes_value(true)
            .default_value("100"))
        .arg(Arg::with_name("FILE")
            .help("Path to image file")
            .required(true))
        .get_matches();

    let filepath = String::from(matches.value_of("FILE").unwrap());
    let n = matches.value_of("n").unwrap().parse::<u32>().unwrap();
    let t = match matches.value_of("shape").unwrap().to_lowercase().as_ref() {
        "triangle" => primg::ShapeType::Triangle,
        _ => panic!("invalid shape"),
    };

    let config = primg::Config { filepath, n, t };
    primg::run(config);
}
