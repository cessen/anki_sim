extern crate rand;

const DAYS: usize = 500;
const NEW_CARDS_PER_DAY: usize = 100;
const MATURE_DAYS: usize = 2;
const REVIEW_VS_NEW_TIME_RATIO: f64 = 20.0 / 60.0;
const STANDARD_RETENTION_RATIO: f32 = 0.9;
const MAX_LAPSES: usize = 8;

fn main() {
    let mut interval_factor = 1.75;
    while interval_factor < 10.0 {
        interval_factor += 0.25;
        let retention_ratio = (interval_factor / 2.5 * STANDARD_RETENTION_RATIO.ln()).exp();
        let is_remembered = || -> bool { rand::random::<f32>() < retention_ratio };

        let mut deck: Vec<Card> = Vec::new();
        let mut new_learn_time = 0.0;
        let mut review_time = 0.0;

        for _ in 0..DAYS {
            // Do "reviews" (forget cards and update stats).
            let mut i = 0;
            while i < deck.len() {
                if deck[i].days_since_last_review >= deck[i].interval {
                    // Do review.
                    review_time += REVIEW_VS_NEW_TIME_RATIO;
                    if is_remembered() {
                        deck[i].interval *= interval_factor;
                        deck[i].interval += (rand::random::<f32>() - 0.5) * deck[i].interval * 0.2;
                        deck[i].days_since_last_review = 0.0;
                    } else if deck[i].lapses < MAX_LAPSES {
                        // deck[i].interval *= 0.5;
                        deck[i].interval /= interval_factor;
                        // deck[i].interval = 1.0;
                        deck[i].days_since_last_review = 0.0;
                        deck[i].lapses += 1;
                    } else {
                        deck.swap_remove(i);
                        continue;
                    }
                } else {
                    deck[i].days_since_last_review += 1.0;
                }
                i += 1;
            }

            // Add new cards
            for _ in 0..NEW_CARDS_PER_DAY {
                new_learn_time += 1.0;
                deck.push(Card {
                    interval: 1.0,
                    days_since_last_review: 0.0,
                    lapses: 0,
                });
            }
        }

        // Find out and print how many mature cards we have.
        let mut count = 0;
        for card in &deck {
            if card.interval >= MATURE_DAYS as f32 {
                count += 1;
            }
        }

        // Normalize for time spent.
        let count_normalized = 1000.0 * count as f64 * new_learn_time
            / ((new_learn_time + review_time) * DAYS as f64 * NEW_CARDS_PER_DAY as f64);

        println!(
            "Mod: {:.2}  |  Learned: {:.2}  |  (Reviews per new: {:.3})  |  (Retention ratio: {:.3})",
            interval_factor,
            count_normalized,
            review_time / new_learn_time,
            retention_ratio,
        );
    }
}

#[derive(Debug, Copy, Clone)]
struct Card {
    interval: f32,
    days_since_last_review: f32,
    lapses: usize,
}
