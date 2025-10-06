use crate::safe::sort::Sorter;

pub struct QuickSorter;

fn quick_sort<T: Ord>(slice: &mut [T]) {
    if slice.len() <= 1 {
        return;
    }

    let pivot_index = slice.len() >> 1;

    slice.swap(pivot_index, slice.len() - 1);
    let (rest, pivot) = slice.split_at_mut(slice.len() - 1);
    let pivot = &pivot[0];
    let mut left = 0;
    let mut right = rest.len();

    while left < right {
        if &rest[left] <= pivot {
            left += 1;
        } else if &rest[right - 1] > pivot {
            right -= 1;
        } else {
            rest.swap(left, right - 1);
            left += 1;
            right -= 1;
        }
    }

    slice.swap(slice.len() - 1, left);
    quick_sort(&mut slice[..left]);
    quick_sort(&mut slice[left + 1..]);
}

impl<T> Sorter<T> for QuickSorter {
    fn sort(&self, slice: &mut [T])
    where
        T: Ord,
    {
        quick_sort(slice);
    }
}

#[test]
fn it_works() {
    let sorter = QuickSorter;
    let mut vec = [5, 3, 4, 1, 2];
    sorter.sort(&mut vec);
    assert_eq!(vec, [1, 2, 3, 4, 5]);
}

#[test]
fn it_works_empty() {
    let sorter = QuickSorter;
    let mut vec: [i32; 0] = [];
    sorter.sort(&mut vec);
    assert_eq!(vec, []);
}

#[test]
fn test_huge() {
    let sorter = QuickSorter;
    let mut vec: Vec<i32> = (0..10000).rev().collect();
    sorter.sort(&mut vec);
    assert_eq!(vec, (0..10000).collect::<Vec<_>>());
}
