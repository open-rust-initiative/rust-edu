use std::f64::{INFINITY, NEG_INFINITY};

#[derive(Clone, Copy)]
pub struct Interval {
    pub min: f64,
    pub max: f64,
}

impl Default for Interval {
    fn default() -> Self {
        Self { min: Default::default(), max: Default::default() }
    }
}

impl Interval {
    pub fn new() -> Interval {
        Interval {
            min: INFINITY,
            max: NEG_INFINITY,
        }
    }

    pub fn with_value(num: f64) -> Interval {
        Interval { min: num, max: num }
    }

    pub fn new_with_values(min: f64, max: f64) -> Interval {
        Interval { min, max }
    }

    pub fn new_with_intervals(a: &Interval, b: &Interval) -> Interval {
        Interval {
            min: a.min.min(b.min),
            max: a.max.max(b.max),
        }
    }

    pub fn size(&self) -> f64 {
        self.max - self.min
    }

    pub fn expand(&self, delta: f64) -> Interval {
        let padding = delta / 2.0;
        Interval {
            min: self.min - padding,
            max: self.max + padding,
        }
    }

    pub fn average(&self) -> f64 {
        (self.min + self.max) / 2.0
    }

    pub fn contains(&self, x: f64) -> bool {
        self.min <= x && x <= self.max
    }

    pub fn surrounds(&self, x: f64) -> bool {
        self.min < x && x < self.max
    }

    pub fn outside(&self, x: f64) -> bool {
        self.min > x || x > self.max
    }

    pub fn clamp(&self, x: f64) -> f64 {
        if x < self.min {
            self.min
        } else if x > self.max {
            self.max
        } else {
            x
        }
    }
}

impl PartialOrd for Interval {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.min < other.min {
            Some(std::cmp::Ordering::Less)
        } else if self.min > other.min {
            Some(std::cmp::Ordering::Greater)
        } else if self.max < other.max {
            Some(std::cmp::Ordering::Less)
        } else if self.max > other.max {
            Some(std::cmp::Ordering::Greater)
        } else {
            Some(std::cmp::Ordering::Equal)
        }
    }
}

impl PartialEq for Interval {
    fn eq(&self, other: &Self) -> bool {
        self.min == other.min && self.max == other.max
    }
}

impl Eq for Interval {}

const EMPTY: Interval = Interval {
    min: INFINITY,
    max: NEG_INFINITY,
};

const UNIVERSE: Interval = Interval {
    min: NEG_INFINITY,
    max: INFINITY,
};

impl std::ops::Add<f64> for Interval {
    type Output = Interval;

    fn add(self, displacement: f64) -> Interval {
        Interval {
            min: self.min + displacement,
            max: self.max + displacement,
        }
    }
}

impl std::ops::Add<Interval> for f64 {
    type Output = Interval;

    fn add(self, interval: Interval) -> Interval {
        interval + self
    }
}