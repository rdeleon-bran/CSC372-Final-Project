/*File:        main.rs
  Author:      Trinity Adams, Rodrigo De Leon Bran
  Course:      CSC372
  Assignment:  Final Project Part 2
  Instructor:  Lester McCann
  TA:          Muaz, Daniel
  Due Date:    4/20/2026

  Description:
  This program reads an ASCII PGM (P2) image and performs image segmentation
  using an iterative thresholding algorithm. It calculates a threshold by
  splitting pixel values into two groups and refining the average until it
  converges. The result is written as a binary PBM image.

  Operational Notes:
  - Run with: cargo run <image.pgm>
  - Only supports ASCII PGM format
  - Threshold may vary slightly due to random sampling
*/

use std::env;
use std::process;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::io::{BufRead, BufReader};
use rand::prelude::*;

// pgm structure
// Stores image dimensions, pixel values, and comment lines from the file
struct PGM{
    width: usize,   // number of columns in the image
    height: usize,  // number of rows in the image
    pixels: Vec<f64>, // grayscale pixel values
    comments: Vec<String> // comment lines from the file
}

// reading the pgm image and saving its values
// reads a P2 file and returns a populated PGM struct
// assumes file exists and is properly formatted
fn read_pgm(path: &str) -> PGM {
    let file = File::open(path).expect("Failed to open file.");
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let init_val = lines.next().unwrap().unwrap();
    assert!(init_val == "P2", "Only ASCII PGM");

    let mut comments = Vec::new();
    let mut comm_lines = lines.next().unwrap().unwrap();
    while comm_lines.starts_with('#') {
        comments.push(comm_lines.clone());
        comm_lines = lines.next().unwrap().unwrap();
    }

    let cline: Vec<_> = comm_lines.split_whitespace().collect();
    let width: usize = cline[0].parse().unwrap();
    let height: usize = cline[1].parse().unwrap();

    let _max_val: f64 = lines.next().unwrap().unwrap().parse().unwrap();

    let mut pixels = Vec::new();
    for line in lines {
        for val in line.unwrap().split_whitespace() {
            pixels.push(val.parse::<f64>().unwrap());
        }
    }

    PGM { width, height, pixels, comments }
}

// calculating the pixels of the grayscale image to perform thresholding
// uses an iterative method to compute a stable threshold value
// returns the final threshold used for segmentation
fn calc_thresh(pixels: &[f64]) -> f64{
    let mut rng = rand::rng();

    // initial random sample used to estimate starting threshold
    let init: Vec<_> = pixels.sample(&mut rng, 10).cloned().collect();
    let sum: f64 = init.iter().sum();
    let count = init.len() as f64;
    let mut threshold = sum / count;

    let mut prev_threshold;

    for _ in 0..100{
        prev_threshold = threshold;

        let mut sum1 = 0.0;   // sum of pixels below threshold
        let mut sum2 = 0.0;   // sum of pixels above threshold
        let mut count1 = 0.0; // number of pixels below threshold
        let mut count2 = 0.0; // number of pixels above threshold

        for &p in pixels {
            if p < threshold{
                sum1 += p;
                count1 += 1.0;
            } else {
                sum2 += p;
                count2 += 1.0;
            }
        }

        let mean1;
        let mean2;

        if count1 > 0.0{
            mean1 = sum1 / count1;
        } else {
            mean1 = 0.0;
        }

        if count2 > 0.0{
            mean2 = sum2 / count2;
        } else {
            mean2 = 0.0;
        }

        threshold = (mean1 + mean2) / 2.0;

        // stop if threshold stabilizes
        if (threshold - prev_threshold).abs() < 0.001 {
            break;
        }
    }

    threshold
}

// created the segmentation of the pixels
// converts grayscale pixels into binary values based on threshold
fn segmentation(pixels: &[f64], threshold: f64) -> Vec<u8>{
    let mut result = Vec::with_capacity(pixels.len());

    for &p in pixels{
        if p < threshold{
            result.push(1);
        } else {
            result.push(0);
        }
    }
    result
}

// creating a new file with the same name and a pbm extension
// writes the segmented binary image to a PBM (P1) file
fn write_pgm(path: &str, width: usize, height: usize, pixels: &[u8], comments: &[String]){
    let out_path = Path::new(path).with_extension("pbm");
    let mut file = File::create(out_path).expect("Failed to create file");

    writeln!(file, "P1").unwrap();

    for c in comments {
        writeln!(file, "{}", c).unwrap();
    }

    writeln!(file, "{} {}", width, height).unwrap();

    for (i, &p) in pixels.iter().enumerate(){
        let bit;
        if p == 0{
            bit = 0;
        } else {
            bit = 1;
        }
        write!(file, "{} ", bit).unwrap();

        if (i + 1) % width == 0{
            writeln!(file).unwrap();
        }
    }

}

// main driver of the program
// handles input argument, processes image, and writes output
fn main(){
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <image.pgm>", args[0]);
        process::exit(1);
    }

    let input_path = &args[1];

    let pgm = read_pgm(input_path);

    let threshold = calc_thresh(&pgm.pixels);

    let segment = segmentation(&pgm.pixels, threshold);

    println!("Threshold: {}", threshold);

    write_pgm(input_path, pgm.width, pgm.height, &segment, &pgm.comments);
}