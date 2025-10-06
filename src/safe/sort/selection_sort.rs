use crate::safe::sort::Sorter;

pub struct SelectionSorter;

impl<T> Sorter<T> for SelectionSorter {
    fn sort(&self, slice: &mut [T])
    where
        T: Ord,
    {
        for unsorted in 0..slice.len() {
            let smallest = slice[unsorted..]
                .iter()
                .enumerate()
                .min_by_key(|&(_, v)| v)
                .map(|(i, _)| unsorted + i)
                .expect("slice is non-empty");

            slice.swap(unsorted, smallest);
        }
    }
}

#[test]
fn it_works() {
    let sorter = SelectionSorter;
    let mut vec = [5, 3, 4, 1, 2];
    sorter.sort(&mut vec);
    assert_eq!(vec, [1, 2, 3, 4, 5]);
}
