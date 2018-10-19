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
    days_past: u32,
    review_count: u32,
    lapse_count: u32,
    remove_lapse_count: u32,

    // Auto-calculated settings
    retention_ratio: f32, // Determined by interval_factor and measured_retention.

    // Settings
    interval_factor: f32, // Multiplier for card intervals on "good" answer.
    lapse_interval_factor: f32, // Multiplier for card intervals on "again" answer.
    measured_retention: (f32, f32), // Retention ratio, interval factor of that ratio
    difficulty_variance: f32,
    max_lapses: u32,
    time_per_new_card: f32,
    time_per_review_card: f32,
    time_per_lapsed_card: f32,
}

impl AnkiSim {
    pub fn new() -> Self {
        AnkiSim {
            deck: Vec::new(),
            cards_added: 0,
            time_spent_on_new: 0.0,
            time_spent_on_review: 0.0,
            days_past: 0,
            review_count: 0,
            lapse_count: 0,
            remove_lapse_count: 0,

            retention_ratio: 0.9,

            interval_factor: 2.5,
            lapse_interval_factor: 0.0,
            measured_retention: (0.9, 2.5),
            difficulty_variance: 0.05,
            max_lapses: 8,
            time_per_new_card: 90.0,
            time_per_review_card: 20.0,
            time_per_lapsed_card: 40.0,
        }
    }

    pub fn average_retention_ratio(&self) -> f32 {
        self.retention_ratio
    }

    fn set_retention_ratio(&mut self) {
        self.retention_ratio = (self.interval_factor / self.measured_retention.1
            * self.measured_retention.0.ln()).exp();
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

    pub fn with_measured_retention_ratio(self, ratio: f32, interval: f32) -> Self {
        let mut tmp = self;
        tmp.measured_retention = (ratio, interval);
        tmp.set_retention_ratio();
        tmp
    }

    pub fn with_max_lapses(self, lapses: u32) -> Self {
        let mut tmp = self;
        tmp.max_lapses = lapses;
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

    /// Average number of seconds spent on each review.
    pub fn with_seconds_per_lapsed_card(self, time: f32) -> Self {
        let mut tmp = self;
        tmp.time_per_lapsed_card = time;
        tmp
    }

    /// Adds N new cards to the deck.
    pub fn add_new_cards(&mut self, n: u32) {
        for _ in 0..n {
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

    /// Simulates a single day.
    pub fn simulate_day(&mut self) {
        self.days_past += 1;

        // Do scheduled reviews.
        let mut i = 0;
        while i < self.deck.len() {
            if self.deck[i].days_since_last_review >= self.deck[i].interval {
                // Do review.
                self.review_count += 1;
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
                    self.lapse_count += 1;
                    self.time_spent_on_review += self.time_per_lapsed_card;
                } else {
                    // Lapsed past max lapses
                    self.deck.swap_remove(i);
                    self.lapse_count += 1;
                    self.remove_lapse_count += 1;
                    continue;
                }
            } else {
                // Not scheduled for review today.
                self.deck[i].days_since_last_review += 1.0;
            }
            i += 1;
        }
    }

    /// Simulates multiple days.
    pub fn simulate_n_days(&mut self, n: u32, new_card_per_day: u32) {
        for _ in 0..n {
            self.add_new_cards(new_card_per_day);
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

    pub fn known_cards(&self) -> u32 {
        let retained = -(1.0 - self.retention_ratio) / self.retention_ratio.ln();
        (self.deck.len() as f64 * retained as f64) as u32
    }

    /// Calculates the number of cards learned per hour spent on reviews and new cards.
    pub fn cards_learned_per_hour(&self) -> f32 {
        let count = self.known_cards();
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

    pub fn lapses_per_review(&self) -> f32 {
        self.lapse_count as f32 / self.review_count as f32
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
