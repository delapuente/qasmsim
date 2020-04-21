use std::collections::HashMap;

use crate::statevector::StateVector;

pub type Histogram = HashMap<String, Vec<(u64, usize)>>;

#[derive(Debug, Clone, PartialEq)]
pub struct Computation {
  pub statevector: StateVector,
  pub memory: HashMap<String, u64>,
  pub probabilities: Vec<f64>,
  pub histogram: Option<Histogram>
}

impl Computation {
  pub fn new(memory: HashMap<String, u64>, statevector: StateVector, histogram: Option<Histogram>) -> Self {
    Computation {
      probabilities: statevector.probabilities(),
      statevector,
      memory,
      histogram
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HistogramBuilder {
  histogram: Histogram
}

impl HistogramBuilder {
  pub fn new() -> Self {
    Default::default()
  }

  pub fn update(&mut self, memory: &HashMap<String, u64>) {
    for (key, current_value) in memory {
      if !self.histogram.contains_key(key) {
        self.histogram.insert(key.clone(), Vec::new());
      }
      let values = self.histogram.get_mut(key).expect("get values for key");
      match values.binary_search_by_key(&current_value, |(v, _)| v) {
        Err(idx) => values.insert(idx, (*current_value, 1)),
        Ok(found) => values[found].1 += 1
      }
    }
  }

  pub fn histogram(self) -> Histogram {
    self.histogram
  }
}

#[cfg(test)]
mod test {
  use std::iter::FromIterator;

  use super::*;

  #[test]
  fn test_histogram_builder_empty_histogram() {
    let builder = HistogramBuilder::new();
    let histogram = builder.histogram();
    assert_eq!(histogram, HashMap::new());
  }

  #[test]
  fn test_histogram_builder_one_update() {
    let mut builder = HistogramBuilder::new();
    builder.update(&HashMap::from_iter(vec![
      ("a".into(), 1)
    ]));
    let histogram = builder.histogram();
    assert_eq!(histogram, HashMap::from_iter(vec![
      ("a".into(), vec![(1, 1)])
    ]));
  }

  #[test]
  fn test_histogram_builder_couple_of_updates() {
    let mut builder = HistogramBuilder::new();
    builder.update(&HashMap::from_iter(vec![
      ("a".into(), 1)
    ]));
    builder.update(&HashMap::from_iter(vec![
      ("a".into(), 1)
    ]));
    let histogram = builder.histogram();
    assert_eq!(histogram, HashMap::from_iter(vec![
      ("a".into(), vec![(1, 2)])
    ]));
  }

  #[test]
  fn test_histogram_builder_couple_of_registers() {
    let mut builder = HistogramBuilder::new();
    builder.update(&HashMap::from_iter(vec![
      ("a".into(), 1)
    ]));
    builder.update(&HashMap::from_iter(vec![
      ("b".into(), 1)
    ]));
    let histogram = builder.histogram();
    assert_eq!(histogram, HashMap::from_iter(vec![
      ("a".into(), vec![(1, 1)]),
      ("b".into(), vec![(1, 1)])
    ]));
  }

  #[test]
  fn test_histogram_builder_different_values() {
    let mut builder = HistogramBuilder::new();
    builder.update(&HashMap::from_iter(vec![
      ("a".into(), 5)
    ]));
    builder.update(&HashMap::from_iter(vec![
      ("b".into(), 4)
    ]));
    builder.update(&HashMap::from_iter(vec![
      ("a".into(), 3)
    ]));
    builder.update(&HashMap::from_iter(vec![
      ("b".into(), 2)
    ]));
    let histogram = builder.histogram();
    assert_eq!(histogram, HashMap::from_iter(vec![
      ("a".into(), vec![(3, 1), (5, 1)]),
      ("b".into(), vec![(2, 1), (4, 1)])
    ]));
  }

  #[test]
  fn test_histogram_builder_different_repeated_values() {
    let mut builder = HistogramBuilder::new();
    builder.update(&HashMap::from_iter(vec![
      ("a".into(), 5)
    ]));
    builder.update(&HashMap::from_iter(vec![
      ("b".into(), 4)
    ]));
    builder.update(&HashMap::from_iter(vec![
      ("a".into(), 5)
    ]));
    builder.update(&HashMap::from_iter(vec![
      ("b".into(), 2)
    ]));
    let histogram = builder.histogram();
    assert_eq!(histogram, HashMap::from_iter(vec![
      ("a".into(), vec![(5, 2)]),
      ("b".into(), vec![(2, 1), (4, 1)])
    ]));
  }
}