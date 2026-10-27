#![allow(dead_code)]

use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct HistoryBuffer {
    capacity: usize,
    points: VecDeque<(f64, f64)>,
}

impl HistoryBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            points: VecDeque::with_capacity(capacity.max(1)),
        }
    }

    pub fn push(&mut self, x: f64, y: f64) {
        if self.points.len() == self.capacity {
            self.points.pop_front();
        }
        self.points.push_back((x, y));
    }

    pub fn points(&self) -> &VecDeque<(f64, f64)> {
        &self.points
    }

    pub fn rolling_points(&self) -> Vec<[f64; 2]> {
        let origin = self.points.front().map(|point| point.0).unwrap_or(0.0);
        self.points.iter().map(|(x, y)| [x - origin, *y]).collect()
    }

    pub fn clear(&mut self) {
        self.points.clear();
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.points.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::HistoryBuffer;

    #[test]
    fn keeps_capacity() {
        let mut history = HistoryBuffer::new(2);
        history.push(1.0, 10.0);
        history.push(2.0, 20.0);
        history.push(3.0, 30.0);
        assert_eq!(history.len(), 2);
        assert_eq!(history.points().front().copied(), Some((2.0, 20.0)));
    }
}
