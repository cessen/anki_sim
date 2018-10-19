/// Anki sim using analytics to run faster, but accounting
/// for fewer things.
pub struct AnkiSim {
    // State
    deck: Vec<CardCluster>,
    cards_added: f64,
    days_past: u32,
    time_spent_on_new: f64,
    time_spent_on_review: f64,
    review_count: f64,
    lapse_count: f64,
    remove_lapse_count: f64,

    // Auto-calculated settings
    retention_ratio: f32, // Determined by interval_factor and measured_retention.

    // Settings
    interval_factor: f32, // Multiplier for card intervals on "good" answer.
    lapse_interval_factor: f32, // Multiplier for card intervals on "again" answer.
    measured_retention: (f32, f32), // Retention ratio, interval factor of that ratio
    kill_interval: f32,
    max_lapses: u32,
    time_per_new_card: f32,
    time_per_review_card: f32,
    time_per_lapsed_card: f32,
}

impl AnkiSim {
    pub fn new() -> Self {
        AnkiSim {
            deck: Vec::new(),
            cards_added: 0.0,
            days_past: 0,
            time_spent_on_new: 0.0,
            time_spent_on_review: 0.0,
            review_count: 0.0,
            lapse_count: 0.0,
            remove_lapse_count: 0.0,

            retention_ratio: 0.9,

            interval_factor: 2.5,
            lapse_interval_factor: 0.0,
            measured_retention: (0.9, 2.5),
            kill_interval: 365.0,
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

    pub fn with_difficulty_variance(self, _variance: f32) -> Self {
        self
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

    /// Simulates a single day.
    fn simulate_day(&mut self) {
        self.days_past += 1;

        // Do scheduled reviews.
        let mut i = 0;
        let deck_size = self.deck.len();
        while i < deck_size {
            if self.deck[i].days_since_last_review >= self.deck[i].interval {
                // Update review stats.
                self.review_count += self.deck[i].card_count as f64;
                self.time_spent_on_review +=
                    self.time_per_review_card as f64 * self.deck[i].card_count;

                // Calculate things needed for the good and lapse cards
                let good_interval = self.deck[i].interval * self.interval_factor;
                let lapse_interval = (self.deck[i].interval * self.lapse_interval_factor).max(1.0);
                let good_card_count = self.deck[i].card_count * self.retention_ratio as f64;
                let lapse_card_count = self.deck[i].card_count - good_card_count;

                // Update good.
                self.deck[i].interval = good_interval;
                self.deck[i].card_count = good_card_count;
                self.deck[i].days_since_last_review = 0.0;

                // Add lapse.
                if self.deck[i].lapses < self.max_lapses {
                    // Normal lapse
                    let lapse_card = CardCluster {
                        interval: lapse_interval,
                        days_since_last_review: 0.0,
                        lapses: self.deck[i].lapses + 1,
                        card_count: lapse_card_count,
                    };
                    self.deck.push(lapse_card);
                    self.lapse_count += lapse_card_count;
                    self.time_spent_on_review +=
                        self.time_per_lapsed_card as f64 * lapse_card_count;
                } else {
                    // Lapsed past max lapses.
                    self.lapse_count += lapse_card_count;
                    self.remove_lapse_count += lapse_card_count;
                }
            } else {
                // Not scheduled for review today.
                self.deck[i].days_since_last_review += 1.0;
            }
            i += 1;
        }
    }

    /// Simulates multiple days.
    pub fn simulate_n_days(&mut self, n: u32) {
        if self.cards_added == 0.0 {
            self.deck.push(CardCluster {
                interval: 1.0,
                days_since_last_review: 0.0,
                lapses: 0,
                card_count: 1.0,
            })
        }

        let mut review_time_acc = 0.0;
        let mut review_count_acc = 0.0;
        let mut lapse_count_acc = 0.0;
        let mut remove_lapse_count_acc = 0.0;

        for _ in 0..n {
            self.cards_added += 1.0;
            self.time_spent_on_new += self.time_per_new_card as f64;
            self.simulate_day();
            review_time_acc += self.time_spent_on_review;
            review_count_acc += self.review_count;
            lapse_count_acc += self.lapse_count;
            remove_lapse_count_acc += self.remove_lapse_count;
        }

        self.time_spent_on_review = review_time_acc;
        self.review_count = review_count_acc;
        self.lapse_count = lapse_count_acc;
        self.remove_lapse_count = remove_lapse_count_acc;
    }

    pub fn known_cards(&self) -> f64 {
        let retained = -(1.0 - self.retention_ratio) / self.retention_ratio.ln();
        (self.cards_added - self.remove_lapse_count) * retained as f64
    }

    /// Calculates the number of cards learned per hour spent on reviews and new cards.
    pub fn cards_learned_per_hour(&self) -> f32 {
        (self.known_cards() / (self.time_spent_on_new + self.time_spent_on_review) * 3600.0) as f32
    }

    /// In hours.
    pub fn review_time(&self) -> f32 {
        (self.time_spent_on_review / 3600.0) as f32
    }

    /// In hours.
    pub fn new_time(&self) -> f32 {
        (self.time_spent_on_new / 3600.0) as f32
    }

    pub fn lapses_per_review(&self) -> f32 {
        (self.lapse_count / self.review_count) as f32
    }
}

#[derive(Debug, Copy, Clone)]
struct CardCluster {
    interval: f32,
    days_since_last_review: f32,
    lapses: u32,
    card_count: f64,
}

impl CardCluster {
    fn new() -> Self {
        Self {
            interval: 1.0,
            days_since_last_review: 0.0,
            lapses: 0,
            card_count: 1.0,
        }
    }
}
