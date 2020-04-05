#[allow(dead_code)]

pub fn interleave<T: Copy>(slice: &[T], stride: usize) -> Vec<T> {
    assert!(slice.len() % stride == 0);
    let mut interleaved = Vec::with_capacity(slice.len());
    let stride_len = slice.len() / stride;
    for i in 0..stride_len {
        for j in 0..stride {
            interleaved.push(slice[i + j * stride_len]);
        }
    }
    interleaved
}

pub fn deinterleave<T: Copy>(slice: &[T], stride: usize) -> Vec<T> {
    assert!(slice.len() % stride == 0);
    let mut deinterleaved = Vec::with_capacity(slice.len());
    let stride_len = slice.len() / stride;
    for i in 0..stride {
        for j in 0..stride_len {
            deinterleaved.push(slice[i + stride * j]);
        }
    }
    deinterleaved
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_interleave() {
        let data = vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9]; 
        assert_eq!(
            interleave(&data, 2),
            [0.0, 0.5, 0.1, 0.6, 0.2, 0.7, 0.3, 0.8, 0.4, 0.9]
        );
    }

    #[test]
    fn test_deinterleave() {
        let data = vec![0.0, 0.5, 0.1, 0.6, 0.2, 0.7, 0.3, 0.8, 0.4, 0.9]; 
        assert_eq!(
            deinterleave(&data, 2),
            [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9]
        );
    }
}
