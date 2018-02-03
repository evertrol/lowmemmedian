extern crate env_logger;
extern crate lowmemmedian;
extern crate time;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::exit;
use time::PreciseTime;

fn main() {
    env_logger::init();

    let args: Vec<_> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: {} <nlines> <data-file>", args[0]);
        exit(1);
    }
    let ndata: usize = args[1].parse().unwrap();
    let file = File::open(args[2].as_str()).expect("File not found");
    let file = BufReader::new(file);

    let mut data: Vec<f64> = Vec::with_capacity(ndata);
    for (i, line) in file.lines().enumerate() {
        if let Ok(curline) = line {
            data.push(curline.parse().unwrap());
        } else {
            println!("error at line {}", i + 1);
        }
        if data.len() >= ndata {
            break;
        }
    }

    println!("Read {} lines correctly", data.len());
    data.truncate(ndata);
    let start = PreciseTime::now();
    let median = lowmemmedian::calcgen(&data, 10.0, 0.2, 0.5);
    let duration = start.to(PreciseTime::now());
    let microsecs = duration.num_microseconds().unwrap();
    let seconds = (microsecs as f64) / 1e6;
    println!("Median (duration) = {:.10} ( {} sec.)", median, seconds);
}
