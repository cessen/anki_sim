#![allow(dead_code)]

extern crate png_encode_mini;
extern crate rand;

mod anki_sim;

use std::fs::File;
use std::io::Write;

fn main() {
    // generate_chart("yar.png", 10, false, (2.0, 19.75), 18 * 4, (0.01, 1.0), 100);
    print_vertical_slice(100, (2.0, 10.0), 33, 0.9);
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
            .with_difficulty_variance(0.0)
            .with_max_lapses(999999)
            .with_new_cards_per_day(samples)
            .with_seconds_per_new_card(60.0 * 3.0)
            .with_seconds_per_review_card(20.0)
            .with_seconds_per_lapsed_card(40.0);

        anki.simulate_n_days(365);

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
                .with_new_cards_per_day(samples)
                .with_seconds_per_new_card(120.0)
                .with_seconds_per_review_card(20.0);

            anki.simulate_n_days(365);
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

    // Create the image
    let mut image = vec![0u8; height * width * 4];
    for i in 0..(height * width) {
        let val = (chart[i] * 255.0) as u8;
        image[i * 4] = val;
        image[i * 4 + 1] = val;
        image[i * 4 + 2] = val;
        image[i * 4 + 3] = 255;
    }

    // Write the image
    png_encode_mini::write_rgba_from_u8(
        &mut File::create(path).unwrap(),
        &image,
        width as u32,
        height as u32,
    ).unwrap();
}
