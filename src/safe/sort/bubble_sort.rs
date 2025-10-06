use crate::safe::sort::Sorter;

pub struct BubbleSorter;

impl<T> Sorter<T> for BubbleSorter {
    fn sort(&self, slice: &mut [T])
    where
        T: Ord,
    {
        let mut swapped = true;
        while swapped {
            swapped = false;
            for i in 1..slice.len() {
                if slice[i - 1] > slice[i] {
                    slice.swap(i - 1, i);
                    swapped = true;
                }
            }
        }
    }
}

#[test]
fn it_works() {
    let sorter = BubbleSorter;
    let mut vec = [5, 3, 4, 1, 2];
    sorter.sort(&mut vec);
    assert_eq!(vec, [1, 2, 3, 4, 5]);
}
