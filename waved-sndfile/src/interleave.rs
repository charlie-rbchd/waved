pub trait SliceInterleave {
    fn interleave(&mut self);
    fn deinterleave(&mut self);
}

impl<T> SliceInterleave for [T]
where
    T: Copy
{
    fn interleave(&mut self) {
        assert!(self.len() >= 2);
        inshuffle_permutation(self, 1, self.len() - 1);
    }

    fn deinterleave(&mut self) {
        // FIXME: Adapt inshuffle_permutation to do the opposite operation
    }
}

fn rotate<T>(slice: &mut [T], first: usize, n_first: usize, last: usize) {
    if first == n_first || n_first == last {
        return;
    }
  
    let mut read = n_first;
    let mut write = first;
    let mut next_read = first; // read position for when "read" hits "last"
  
    while read != last {
       if write == next_read {
           next_read = read; // track where "first" went
       }

       slice.swap(write, read);
       write +=1;
       read +=1;
    }
  
    // rotate the remaining sequence into place
    rotate(slice, write, next_read, last);
}

// Translated to rust from https://www.programmercoach.com/2017/04/interview-pearls-interleave-array-in.html
fn inshuffle_permutation<T: Copy>(slice: &mut [T], first: usize, last: usize) {
    let size = last - first;
    if size == 0 || (size & 1) != 0 {
        return
    }

    let mut i = 1;
    while i * 3 <= size + 1 {
        i *= 3; // Largest power of three
    }

    rotate(slice, first + (i - 1) / 2, first + size / 2, first + (i - 1) / 2 + size / 2);

    let mut m = 1;
    while m < i { // Permutation cycles
        let mut tmp1 = slice[first + (m * 2) % i - 1];
        slice[first + (m * 2) % i - 1] = slice[first + m - 1];

        let mut idx = (m * 2) % i;
        while idx != m {
            let tmp2 = slice[first + (idx * 2) % i - 1];
            slice[first + (idx * 2) % i - 1] = tmp1;
            tmp1 = tmp2;
            idx = (idx * 2) % i;
        }
        m *= 3;
    }

    inshuffle_permutation(slice, first + (i - 1), last); // Split and process the remaining elements
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_interleave() {
        let mut data = vec![0.1, 0.1, 0.1, 0.1, 0.1, 0.2, 0.2, 0.2, 0.2, 0.2]; 
        data.interleave();
        assert_eq!(data, [0.1, 0.2, 0.1, 0.2, 0.1, 0.2, 0.1, 0.2, 0.1, 0.2]);
    }

    #[test]
    fn test_deinterleave() {
        let mut data = vec![0.1, 0.2, 0.1, 0.2, 0.1, 0.2, 0.1, 0.2, 0.1, 0.2]; 
        data.deinterleave();
        assert_eq!(data, [0.1, 0.1, 0.1, 0.1, 0.1, 0.2, 0.2, 0.2, 0.2, 0.2]);
    }
}
