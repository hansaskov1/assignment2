use std::{
    thread,
    time::{Duration, Instant},
};

pub trait IntervalIterator: Iterator {
    /// Creates a new `IntervalIter` iterator that yields items from the original iterator
    /// with the specified time interval between them.
    ///
    /// # Arguments
    ///
    /// * `interval_duration` - The time interval to wait between yielding items.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let values = vec![1, 2, 3, 4, 5];
    /// let interval = Duration::from_secs(1);
    ///
    /// let interval_iter = values.into_iter().with_interval(interval);
    ///
    /// // Consume the interval iterator
    /// for value in interval_iter {
    ///     println!("{}", value);
    /// }
    /// ```
    fn with_interval(self, interval: Duration) -> IntervalIter<Self::Item, Self>
    where
        Self: Sized,
    {
        IntervalIter {
            original_iterator: self,
            interval,
        }
    }
}

impl<I: Iterator> IntervalIterator for I {}

pub struct IntervalIter<T, I: Iterator<Item = T>> {
    original_iterator: I,
    interval: Duration,
}

impl<T, I: Iterator<Item = T>> IntervalIter<T, I> {
    pub fn new(original_iterator: I, interval: Duration) -> Self {
        Self {
            original_iterator,
            interval,
        }
    }
}

// Calculate the remaining time to wait before yielding the item
// by subtracting the elapsed time from the specified interval duration
// The `saturating_sub` method ensures that the result is never negative
impl<T, I: Iterator<Item = T>> Iterator for IntervalIter<T, I> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let start = Instant::now();
        let item = self.original_iterator.next()?;
        let elapsed = start.elapsed();
        let remaining_interval = self.interval.saturating_sub(elapsed);
        thread::sleep(remaining_interval);
        Some(item)
    }
}
