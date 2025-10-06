use std::{cell::Cell, rc::Rc};

use crate::safe::sort::{
    bubble_sort::BubbleSorter, insertion_sort::InsertionSorter, quick_sort::QuickSorter,
    selection_sort::SelectionSorter,
};

pub mod bubble_sort;
pub mod insertion_sort;
pub mod quick_sort;
pub mod selection_sort;

pub trait Sorter<T> {
    fn sort(&self, slice: &mut [T])
    where
        T: Ord;
}

pub struct StdSorter;

impl<T> Sorter<T> for StdSorter {
    fn sort(&self, slice: &mut [T])
    where
        T: Ord,
    {
        slice.sort();
    }
}

pub struct StdUnstableSorter;

impl<T> Sorter<T> for StdUnstableSorter {
    fn sort(&self, slice: &mut [T])
    where
        T: Ord,
    {
        slice.sort_unstable();
    }
}

struct SortEvaluator<T> {
    value: T,
    comparisons: Rc<Cell<usize>>,
}
impl<T> PartialEq for SortEvaluator<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.comparisons.set(self.comparisons.get() + 1);
        self.value == other.value
    }
}

impl<T> Eq for SortEvaluator<T> where T: Eq {}

impl<T> PartialOrd for SortEvaluator<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.comparisons.set(self.comparisons.get() + 1);
        self.value.partial_cmp(&other.value)
    }
}

impl<T> Ord for SortEvaluator<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.comparisons.set(self.comparisons.get() + 1);
        self.value.cmp(&other.value)
    }
}

#[test]
fn bench() {
    let comparisons = Rc::new(Cell::new(0));
    let bench = |sorter: &dyn Sorter<_>| {
        let mut slice: Vec<SortEvaluator<i32>> = (0..1000)
            .rev()
            .map(|v| SortEvaluator {
                value: v,
                comparisons: Rc::clone(&comparisons),
            })
            .collect();
        sorter.sort(&mut slice);
        assert!(slice.windows(2).all(|w| w[0] <= w[1]));
        comparisons.get()
    };

    let bubble = bench(&BubbleSorter);
    let selection = bench(&SelectionSorter);
    let insertion = bench(&InsertionSorter);
    let quick = bench(&QuickSorter);
    let std = bench(&StdSorter);
    let std_unstable = bench(&StdUnstableSorter);

    println!("Bubble: {bubble}");
    println!("Selection: {selection}");
    println!("Insertion: {insertion}");
    println!("Quick: {quick}");
    println!("Std: {std}");
    println!("StdUnstable: {std_unstable}");
}
