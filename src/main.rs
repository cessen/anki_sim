#![allow(dead_code)]

extern crate png_encode_mini;
extern crate rand;

mod anki_sim;

use std::fs::File;
use std::io::Write;

fn main() {
    generate_chart("yar.png", 100, false, (2.0, 10.0), 65, (0.000001, 1.0), 101);
    // print_vertical_slice(100, (2.0, 10.0), 33, 0.9);
}

fn print_vertical_slice(
    samples: u32,
    interval_range: (f32, f32),
    interval_cells: u32,
    measured_retention: f32,
) {
    let count = interval_cells as usize;
    let interval_step = (interval_range.1 - interval_range.0) / (interval_cells - 1) as f32;

    for n in 0..count {
        let interval_factor = interval_range.0 + (interval_step * n as f32);
        let mut anki = anki_sim::AnkiSim::new()
            .with_interval_factor(interval_factor)
            .with_measured_retention_ratio(measured_retention, 2.5)
            .with_lapse_interval_factor((1.0 / interval_factor).sqrt())
            .with_difficulty_variance(0.05)
            .with_max_lapses(8)
            .with_seconds_per_new_card(20.0 * 6.0)
            .with_seconds_per_review_card(20.0)
            .with_seconds_per_lapsed_card(20.0 * 2.0);

        anki.simulate_n_days(365, samples);

        println!(
            "Interval Factor: {:.2}  |  Cards learned per hour: {:.2}  |  Lapse ratio: {:.2}",
            interval_factor,
            anki.cards_learned_per_hour(),
            anki.lapses_per_review(),
        );
    }
}

fn generate_chart(
    path: &str,
    samples: u32,
    normalize_whole: bool,
    interval_range: (f32, f32),
    interval_cells: u32,
    retention_range: (f32, f32),
    retention_cells: u32,
) {
    let height = interval_cells as usize;
    let interval_step = (interval_range.1 - interval_range.0) / (interval_cells - 1) as f32;
    let width = retention_cells as usize;
    let retention_step = (retention_range.1 - retention_range.0) / (retention_cells - 1) as f32;

    print!("\n0.0%");
    let _ = std::io::stdout().flush();
    let mut chart = vec![0.0f32; height * width];
    for x in 0..width {
        let retention_ratio = retention_range.0 + (retention_step * x as f32);
        for y in 0..height {
            let interval_factor = interval_range.0 + (interval_step * y as f32);
            let mut anki = anki_sim::AnkiSim::new()
                .with_interval_factor(interval_factor)
                .with_measured_retention_ratio(retention_ratio, 2.5)
                .with_lapse_interval_factor((1.0 / interval_factor).sqrt())
                .with_difficulty_variance(0.05)
                .with_max_lapses(8)
                .with_seconds_per_new_card(20.0 * 6.0)
                .with_seconds_per_review_card(20.0)
                .with_seconds_per_lapsed_card(20.0 * 2.0);

            anki.simulate_n_days(365, samples);
            chart[y * width + x] = anki.cards_learned_per_hour();

            print!(
                "\r{:.1}%",
                (x * height + y) as f32 / (height * width) as f32 * 100.0
            );
            let _ = std::io::stdout().flush();
        }
    }
    println!("\nDone.");

    if !normalize_whole {
        // Normalize each column (i.e. each interval within itself).
        for x in 0..width {
            let mut max = 0.0;
            for y in 0..height {
                max = if max > chart[y * width + x] {
                    max
                } else {
                    chart[y * width + x]
                };
            }
            for y in 0..height {
                chart[y * width + x] /= max;
            }
        }
    } else {
        // Normalize across whole chart.
        let max = chart
            .iter()
            .fold(0.0, |acc, n| if acc > *n { acc } else { *n });
        for v in &mut chart {
            *v /= max;
        }
    }

    // Create the image, enlarging by the magnification factor.
    let magnification_factor = 10;
    let image_width = (width - 1) * magnification_factor;
    let image_height = (height - 1) * magnification_factor;
    let width_map_fac = (width - 1) as f64 / (image_width - 1) as f64;
    let height_map_fac = (height - 1) as f64 / (image_height - 1) as f64;
    let mut image = vec![0u8; image_height * image_width * 4];
    for x in 0..image_width {
        for y in 0..image_height {
            let val = {
                let small_x = x as f64 * width_map_fac;
                let small_y = y as f64 * height_map_fac;
                let sx1 = small_x as usize;
                let sy1 = small_y as usize;
                let sx2 = (sx1 + 1).min(width - 1);
                let sy2 = (sy1 + 1).min(height - 1);
                let alpha_x = (small_x - small_x.floor()) as f32;
                let alpha_y = (small_y - small_y.floor()) as f32;

                let left = {
                    let val1 = chart[sy1 * width + sx1];
                    let val2 = chart[sy2 * width + sx1];
                    (val1 * (1.0 - alpha_y)) + (val2 * alpha_y)
                };

                let right = {
                    let val1 = chart[sy1 * width + sx2];
                    let val2 = chart[sy2 * width + sx2];
                    (val1 * (1.0 - alpha_y)) + (val2 * alpha_y)
                };

                let lerped = (left * (1.0 - alpha_x)) + (right * alpha_x);

                (lerped * 255.0) as u8
            };
            let i = y * image_width + x;
            image[i * 4] = val;
            image[i * 4 + 1] = val;
            image[i * 4 + 2] = val;
            image[i * 4 + 3] = 255;
        }
    }

    // Write the image
    png_encode_mini::write_rgba_from_u8(
        &mut File::create(path).unwrap(),
        &image,
        image_width as u32,
        image_height as u32,
    ).unwrap();
}
