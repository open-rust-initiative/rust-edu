use {
    rand::{
        distributions::uniform::SampleUniform, rngs::StdRng, seq::SliceRandom, thread_rng, Rng,
        RngCore, SeedableRng,
    },
    std::f64,
    std::io::{self, Write},
    std::ops::Range,
    std::sync::{Arc, Mutex},
};

// double inf
const INFINITY: f64 = f64::INFINITY;
const PI: f64 = std::f64::consts::PI;
const ESP: f64 = 1e-8;
const ESP3: f64 = ESP * ESP * ESP;

// 角度转弧度
pub(crate) fn degrees_to_radians(degrees: f64) -> f64 {
    degrees * PI / 180.0
}

// 判断 x 的范围是否在 [min, max] 之间 否则现在边界
pub(crate) fn clamp(x: f64, min: f64, max: f64) -> f64 {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}

// 进度条（需要加线程锁）
pub(crate) fn update_progress(progress: f64) {
    const BAR_WIDTH: usize = 100;

    print!("[");
    let pos = (BAR_WIDTH as f64 * progress) as usize;
    for i in 0..BAR_WIDTH {
        if i < pos {
            print!("=");
        } else if i == pos {
            print!(">");
        } else {
            print!(" ");
        }
    }
    print!("] {} %\r", (progress * 100.0) as u32);
    io::stdout().flush().unwrap();
}

pub(crate) fn update_progress_with_lock(now: i32, total: i32, lock: &Arc<Mutex<()>>) {
    let progress = now as f64 / total as f64;
    let _guard = lock.lock().unwrap();
    update_progress(progress);
}

// random

#[must_use]
fn normal<R: Rng>(mut rng: R) -> f64 {
    rng.gen_range(0.0, 1.0)
}

#[must_use]
fn range<R: Rng, T: SampleUniform + PartialOrd>(mut rng: R, r: Range<T>) -> T {
    rng.gen_range(r.start, r.end)
}

fn choose<T, R: Rng, S: AsRef<[T]>>(mut rng: R, values: &S) -> &T {
    let slice = values.as_ref();
    assert!(!slice.is_empty());
    let index = rng.gen_range(0, slice.len());
    &slice[index]
}

fn shuffle<T, R: Rng, S: AsMut<[T]>>(mut rng: R, values: &mut S) {
    let slice = values.as_mut();
    slice.shuffle(&mut rng);
}

#[derive(Debug)]
pub struct Random();

impl Random {
    // Return random number in range [0, 1]
    #[must_use]
    pub fn normal() -> f64 {
        normal(thread_rng())
    }

    #[must_use]
    pub fn range<T: SampleUniform + PartialOrd>(r: Range<T>) -> T {
        range(thread_rng(), r)
    }

    pub fn choose<T, S: AsRef<[T]>>(values: &S) -> &T {
        choose(thread_rng(), values)
    }

    pub fn shuffle<T, S: AsMut<[T]>>(values: &mut S) {
        shuffle(thread_rng(), values)
    }
}

#[derive(Debug)]
pub struct SeedRandom(StdRng);

impl Default for SeedRandom {
    fn default() -> Self {
        Self::random()
    }
}

impl SeedRandom {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self(StdRng::seed_from_u64(seed))
    }

    #[must_use]
    pub fn random() -> Self {
        Self::new(rand::thread_rng().next_u64())
    }

    pub fn normal(&mut self) -> f64 {
        normal(&mut self.0)
    }

    pub fn range<T: SampleUniform + PartialOrd>(&mut self, r: Range<T>) -> T {
        range(&mut self.0, r)
    }

    pub fn choose<'i, 's, T, S: AsRef<[T]>>(&'i mut self, values: &'s S) -> &'s T {
        choose(&mut self.0, values)
    }

    pub fn shuffle<T, S: AsMut<[T]>>(&mut self, values: &mut S) {
        shuffle(&mut self.0, values)
    }
}
