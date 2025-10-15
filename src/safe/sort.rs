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

#[cfg(test)]
mod tests {
    use crate::{
        cell::Cell,
        rc::Rc,
        safe::sort::{
            Sorter, StdSorter, StdUnstableSorter, bubble_sort::BubbleSorter,
            insertion_sort::InsertionSorter, quick_sort::QuickSorter,
            selection_sort::SelectionSorter,
        },
    };

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
            comparisons.set(0);
            let mut slice: Vec<SortEvaluator<i32>> = (0..1000)
                .rev()
                .map(|v| SortEvaluator {
                    value: v,
                    comparisons: Rc::clone(&comparisons),
                })
                .collect();
            let start = std::time::Instant::now();
            sorter.sort(&mut slice);
            let duration = start.elapsed();
            assert!(slice.windows(2).all(|w| w[0] <= w[1]));
            (comparisons.get(), duration)
        };

        let bubble = bench(&BubbleSorter);
        let selection = bench(&SelectionSorter);
        let insertion = bench(&InsertionSorter);
        let quick = bench(&QuickSorter);
        let std = bench(&StdSorter);
        let std_unstable = bench(&StdUnstableSorter);

        println!("Bubble: {} {}", bubble.0, bubble.1.as_nanos());
        println!("Selection: {} {}", selection.0, selection.1.as_nanos());
        println!("Insertion: {} {}", insertion.0, insertion.1.as_nanos());
        println!("Quick: {} {}", quick.0, quick.1.as_nanos());
        println!("Std: {} {}", std.0, std.1.as_nanos());
        println!(
            "StdUnstable: {} {}",
            std_unstable.0,
            std_unstable.1.as_nanos()
        );
    }
}
