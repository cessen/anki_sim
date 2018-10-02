extern crate rand;

mod anki_sim;

const DAYS: usize = 500;

fn main() {
    let mut interval_factor = 1.75;
    while interval_factor < 10.0 {
        interval_factor += 0.25;

        let mut anki = anki_sim::AnkiSim::new()
            .with_interval_factor(interval_factor)
            .with_lapse_interval_factor(0.5)
            .with_typical_retention_ratio(0.85)
            .with_difficulty_variance(0.1)
            .with_max_lapses(3)
            .with_new_cards_per_day(100)
            .with_seconds_per_new_card(60.0 * 3.0)
            .with_seconds_per_review_card(10.0);

        anki.simulate_n_days(DAYS as u32);

        let new_time = anki.new_time();
        let review_time = anki.review_time();
        let tot_time = new_time + review_time;
        println!(
            "Interval: {:.2}  |  Learned per hour: {:.2}  |  New / Reviews: {:.2} / {:.2}",
            interval_factor,
            anki.cards_learned_per_hour(2),
            new_time / tot_time,
            review_time / tot_time,
        );
    }
}
