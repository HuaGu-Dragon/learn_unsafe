pub mod bubble_sort;
pub mod insertion_sort;
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
