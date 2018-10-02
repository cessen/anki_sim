use rand::{
    self,
    distributions::{Distribution, Normal},
    random,
};

pub struct AnkiSim {
    // State
    deck: Vec<Card>,
    cards_added: u32,
    time_spent_on_new: f32,
    time_spent_on_review: f32,

    // Auto-calculated settings
    retention_ratio: f32, // Determined by interval_factor and typical_retention_ratio.

    // Settings
    interval_factor: f32, // Multiplier for card intervals on "good" answer.
    lapse_interval_factor: f32, // Multiplier for card intervals on "again" answer.
    typical_retention_ratio: f32,
    difficulty_variance: f32,
    max_lapses: u32,
    new_cards_per_day: u32, // Only actually matters for statistical accuracy.
    time_per_new_card: f32,
    time_per_review_card: f32,
}

impl AnkiSim {
    pub fn new() -> Self {
        AnkiSim {
            deck: Vec::new(),
            cards_added: 0,
            time_spent_on_new: 0.0,
            time_spent_on_review: 0.0,

            retention_ratio: 0.9,

            interval_factor: 2.5,
            lapse_interval_factor: 0.0,
            typical_retention_ratio: 0.9,
            difficulty_variance: 0.05,
            max_lapses: 8,
            new_cards_per_day: 100,
            time_per_new_card: 90.0,
            time_per_review_card: 20.0,
        }
    }

    fn set_retention_ratio(&mut self) {
        self.retention_ratio =
            (self.interval_factor / 2.5 * self.typical_retention_ratio.ln()).exp();
    }

    pub fn with_difficulty_variance(self, variance: f32) -> Self {
        let mut tmp = self;
        tmp.difficulty_variance = variance;
        tmp
    }

    pub fn with_interval_factor(self, factor: f32) -> Self {
        let mut tmp = self;
        tmp.interval_factor = factor;
        tmp.set_retention_ratio();
        tmp
    }

    pub fn with_lapse_interval_factor(self, factor: f32) -> Self {
        let mut tmp = self;
        tmp.lapse_interval_factor = factor;
        tmp
    }

    pub fn with_typical_retention_ratio(self, ratio: f32) -> Self {
        let mut tmp = self;
        tmp.typical_retention_ratio = ratio;
        tmp.set_retention_ratio();
        tmp
    }

    pub fn with_max_lapses(self, lapses: u32) -> Self {
        let mut tmp = self;
        tmp.max_lapses = lapses;
        tmp
    }

    pub fn with_new_cards_per_day(self, new_cards: u32) -> Self {
        let mut tmp = self;
        tmp.new_cards_per_day = new_cards;
        tmp
    }

    /// Average number of seconds spent on each new card before they become
    /// normal review cards.
    pub fn with_seconds_per_new_card(self, time: f32) -> Self {
        let mut tmp = self;
        tmp.time_per_new_card = time;
        tmp
    }

    /// Average number of seconds spent on each review.
    pub fn with_seconds_per_review_card(self, time: f32) -> Self {
        let mut tmp = self;
        tmp.time_per_review_card = time;
        tmp
    }

    /// Simulates a single day.
    pub fn simulate_day(&mut self) {
        // Do scheduled reviews.
        let mut i = 0;
        while i < self.deck.len() {
            if self.deck[i].days_since_last_review >= self.deck[i].interval {
                // Do review.
                self.time_spent_on_review += self.time_per_review_card;
                if self.deck[i].is_remembered() {
                    // Good
                    self.deck[i].interval *= self.interval_factor;
                    self.deck[i].interval += (random::<f32>() - 0.5) * self.deck[i].interval * 0.2;
                    self.deck[i].days_since_last_review = 0.0;
                } else if self.deck[i].lapses < self.max_lapses {
                    // Normal lapse
                    self.deck[i].interval =
                        (self.deck[i].interval * self.lapse_interval_factor).max(1.0);
                    self.deck[i].days_since_last_review = 0.0;
                    self.deck[i].lapses += 1;
                } else {
                    // Lapsed past max lapses
                    self.deck.swap_remove(i);
                    continue;
                }
            } else {
                // Not scheduled for review today.
                self.deck[i].days_since_last_review += 1.0;
            }
            i += 1;
        }

        // Add new cards.
        for _ in 0..self.new_cards_per_day {
            self.time_spent_on_new += self.time_per_new_card;
            self.cards_added += 1;
            let sampler = Normal::new(self.retention_ratio as f64, self.difficulty_variance as f64);
            self.deck.push(Card {
                interval: 1.0,
                days_since_last_review: 0.0,
                retention_ratio: sampler.sample(&mut rand::thread_rng()).max(0.01).min(0.99) as f32,
                lapses: 0,
            });
        }
    }

    /// Simulates multiple days.
    pub fn simulate_n_days(&mut self, n: u32) {
        for _ in 0..n {
            self.simulate_day();
        }
    }

    /// Number of cards with the given interval or greater.
    fn cards_with_interval_or_greater(&self, interval: u32) -> u32 {
        let mut count = 0;
        for card in &self.deck {
            if card.interval >= interval as f32 {
                count += 1;
            }
        }
        count
    }

    /// Calculates the number of cards learned per time unit spent on reviews and new cards.
    ///
    /// Will only count cards with intervals larger than a given number of days.
    pub fn cards_learned_per_hour(&self, with_intervals_larger_than: u32) -> f32 {
        let count = self.cards_with_interval_or_greater(with_intervals_larger_than);
        count as f32 / (self.time_spent_on_new + self.time_spent_on_review) * 3600.0
    }

    /// In hours.
    pub fn review_time(&self) -> f32 {
        self.time_spent_on_review / 3600.0
    }

    /// In hours.
    pub fn new_time(&self) -> f32 {
        self.time_spent_on_new / 3600.0
    }
}

#[derive(Debug, Copy, Clone)]
struct Card {
    interval: f32,
    days_since_last_review: f32,
    retention_ratio: f32, // Chance that the card will be remembered each review.
    lapses: u32,
}

impl Card {
    fn is_remembered(&self) -> bool {
        random::<f32>() < self.retention_ratio
    }
}
