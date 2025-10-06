use crate::safe::sort::Sorter;

pub struct InsertionSorter;

impl<T> Sorter<T> for InsertionSorter {
    fn sort(&self, slice: &mut [T])
    where
        T: Ord,
    {
        for unsorted in 1..slice.len() {
            let mut i = unsorted;
            while i > 0 && slice[i - 1] > slice[i] {
                slice.swap(i - 1, i);
                i -= 1;
            }
        }
    }
}

#[test]
fn it_works() {
    let sorter = InsertionSorter;
    let mut vec = [5, 3, 4, 1, 2];
    sorter.sort(&mut vec);
    assert_eq!(vec, [1, 2, 3, 4, 5]);
}
