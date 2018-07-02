#[macro_use]
extern crate log;
extern crate rayon;
use rayon::prelude::*;

struct Scalings {
    diff: f64,
    factor: f64,
    maxdiff: f64,
    prevdiff: f64,
    delta: f64,
    decrease: f64,
}

pub fn calccounts(data: &[f64], partition: f64) -> (usize, usize, f64, f64) {
    let mut below = -std::f64::INFINITY;
    let mut above = std::f64::INFINITY;
    let mut nlow = 0;
    let mut nhigh = 0;
    for &value in data.iter() {
        if value <= partition {
            nlow += 1;
        }
        if value >= partition {
            nhigh += 1;
        }
        if value < partition && below < value {
            below = value;
        }
        if value > partition && above > value {
            above = value;
        }
    }
    (nlow, nhigh, below, above)
}

pub fn getcounts(data: &[f64], partition: f64, nchunks: usize) -> (usize, usize, f64, f64) {
    let chunksize = data.len() / nchunks;
    let results: Vec<(usize, usize, f64, f64)> = data
        .par_chunks(chunksize)
        .map(|chunk| calccounts(chunk, partition))
        .collect();

    let mut nlow = 0;
    let mut nhigh = 0;
    let mut below = -std::f64::INFINITY;
    let mut above = std::f64::INFINITY;

    for values in &results {
        nlow += values.0;
        nhigh += values.1;
        if values.2 > below {
            below = values.2;
        }
        if values.3 < above {
            above = values.3;
        }
    }

    (nlow, nhigh, below, above)
}

fn nhigh_nlow(
    scalings: &mut Scalings,
    sign: f64,
    partition: f64,
    prevpartition: &mut f64,
    below: f64,
    above: f64,
) -> f64 {
    if scalings.diff > scalings.maxdiff {
        if scalings.diff > scalings.prevdiff {
            // The change was overestimated
            // Try again with a smaller scaling factor
            // Change `factor` by a minimum of `decrease`, but
            // more if we overestimated the change a lot
            let ratio = scalings.prevdiff / scalings.diff;
            if ratio < scalings.decrease {
                scalings.factor *= ratio;
            } else {
                scalings.factor *= scalings.decrease;
            }
            debug!("Rescaled to factor, ratio = {}, {}", scalings.factor, ratio);
            *prevpartition + scalings.prevdiff * scalings.factor * scalings.delta * sign
        } else {
            scalings.prevdiff = scalings.diff;
            *prevpartition = partition;
            scalings.delta = above - below;
            partition + scalings.diff * scalings.factor * scalings.delta * sign
        }
    } else if sign < 0.0 {
        below
    } else {
        above
    }
}

pub fn calc_final_partition(
    nsame: usize,
    evenlen: bool,
    below: f64,
    above: f64,
    partition: f64,
) -> f64 {
    if nsame > 0 {
        if evenlen && nsame == 1 {
            (below + partition) / 2.0
        } else {
            partition
        }
    } else if evenlen {
        (below + above) / 2.0
    } else {
        below
    }
}

pub fn calculate(data: &[f64], maxdiff: f64, factor: f64, decrease: f64, nchunks: usize) -> f64 {
    let len = data.len();
    if len == 0 {
        return std::f64::NAN;
    }
    if len == 1 {
        return data[0];
    }
    if len == 2 {
        return (data[0] + data[1]) / 2.0;
    }

    let maxdiff = if maxdiff >= 0.0 {
        maxdiff
    } else {
        -maxdiff * len as f64
    };

    let mut scalings = Scalings {
        diff: maxdiff,
        factor,
        maxdiff,
        prevdiff: std::f64::INFINITY,
        delta: 0.0,
        decrease,
    };

    debug!("Initial scalings: factor, decrease = {}, {}", scalings.factor, scalings.decrease);

    let sum: f64 = data.iter().sum();
    let evenlen = len % 2 == 0;
    let mut partition = sum / (len as f64);
    let mut prevpartition = partition;

    debug!(
        "Partition start (= mean), maxdiff: {}, {}",
        partition, maxdiff
    );

    let mut iloop: u64 = 0;
    loop {
        iloop += 1;

        let (nlow, nhigh, below, above) = if nchunks == 1 {
            calccounts(data, partition)
        } else {
            getcounts(data, partition, nchunks)
        };

        // Determine break criteria for the loop, or otherwise the
        // change in partition
        let nsame = nhigh + nlow - len;
        if nlow == nhigh {
            if nsame == 0 {
                partition = (below + above) / 2.0;
            }
            break;
        } else if nlow > nhigh {
            // above the median
            if nlow - nhigh <= nsame {
                partition = calc_final_partition(nsame, evenlen, below, above, partition);
                break;
            }
            scalings.diff = (nlow - nhigh - nsame) as f64;
            let sign = -1.0;
            debug!("diff, nsame = {}, {}", scalings.diff, nsame);
            partition = nhigh_nlow(
                &mut scalings,
                sign,
                partition,
                &mut prevpartition,
                below,
                above,
            );
        } else {
            // below the median
            if nhigh - nlow <= nsame {
                partition = calc_final_partition(nsame, evenlen, above, below, partition);
                break;
            }
            scalings.diff = (nhigh - nlow - nsame) as f64;
            let sign = 1.0;
            debug!("diff, nsame = {}, {}", scalings.diff, nsame);
            partition = nhigh_nlow(
                &mut scalings,
                sign,
                partition,
                &mut prevpartition,
                below,
                above,
            );
        }
        debug!(
            "nlow, nhigh, below, above, delta = {}, {}, {}, {}, {}",
            nlow, nhigh, below, above, scalings.delta
        );
        debug!("iloop, partition: {}, {}", iloop, partition);
    }
    debug!("iloop = {}", iloop);

    partition
}

pub fn calc(data: &[f64]) -> f64 {
    calculate(data, 5.0, 0.2, 0.5, 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    const EPS: f64 = 2.0 * std::f64::EPSILON;

    #[test]
    fn median0() {
        let data: Vec<f64> = vec![];
        let m = calc(&data);
        assert!(m.is_nan());
    }

    #[test]
    fn median1() {
        let data: Vec<f64> = vec![5.0];
        assert!((calc(&data) - 5.0).abs() <= EPS);

        let data = vec![std::f64::INFINITY];
        let m = calc(&data);
        assert_eq!(m, std::f64::INFINITY);
    }

    #[test]
    fn median2() {
        let data: Vec<f64> = vec![5.0, 6.0];
        assert!((calc(&data) - 5.5).abs() <= EPS);

        let data: Vec<f64> = vec![5.0, 5.0];
        assert!((calc(&data) - 5.0).abs() <= EPS);

        let data: Vec<f64> = vec![5.0, std::f64::INFINITY];
        let m = calc(&data);
        assert_eq!(m, std::f64::INFINITY);
    }

    #[test]
    fn median3() {
        let data: Vec<f64> = vec![5.0, 6.0, 7.0];
        assert!((calc(&data) - 6.0).abs() <= EPS);

        let data: Vec<f64> = vec![7.0, 6.0, 5.0];
        assert!((calc(&data) - 6.0).abs() <= EPS);

        let data: Vec<f64> = vec![5.0, 7.0, 6.0];
        assert!((calc(&data) - 6.0).abs() <= EPS);

        let data: Vec<f64> = vec![6.0, 7.0, 5.0];
        assert!((calc(&data) - 6.0).abs() <= EPS);

        let data: Vec<f64> = vec![6.0, 5.0, 7.0];
        assert!((calc(&data) - 6.0).abs() <= EPS);

        let data: Vec<f64> = vec![7.0, 5.0, 6.0];
        assert!((calc(&data) - 6.0).abs() <= EPS);

        let data: Vec<f64> = vec![5.0, 5.0, 7.0];
        assert!((calc(&data) - 5.0).abs() <= EPS);

        let data: Vec<f64> = vec![5.0, 5.0, 5.0];
        assert!((calc(&data) - 5.0).abs() <= EPS);

        let data: Vec<f64> = vec![5.0, 6.0, std::f64::INFINITY];
        assert!((calc(&data) - 6.0).abs() <= EPS);

        let data: Vec<f64> = vec![std::f64::INFINITY, 6.0, std::f64::INFINITY];
        assert_eq!(calc(&data), std::f64::INFINITY);
    }

    #[test]
    fn median() {
        let data: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        assert!((calc(&data) - 4.0).abs() <= EPS);
        let data: Vec<f64> = vec![1.0, 1.0, 1.0, 4.0, 5.0, 6.0, 1.0];
        assert!((calc(&data) - 1.0).abs() <= EPS);
        let data: Vec<f64> = vec![1.0, 1.0, 2.0, 4.0, 5.0, 6.0, 1.0];
        assert!((calc(&data) - 2.0).abs() <= EPS);
        let data: Vec<f64> = vec![4.0, 2.0, 1.0, 7.0, 3.0, 6.0, 5.0];
        assert!((calc(&data) - 4.0).abs() <= EPS);
        let data: Vec<f64> = vec![7.0, 7.0, 1.0, 1.0, 5.0, 4.0, 3.0];
        assert!((calc(&data) - 4.0).abs() <= EPS);
        let data: Vec<f64> = vec![5.0, 3.0, 4.0, 7.0, 1.0, 6.0, 2.0];
        assert!((calc(&data) - 4.0).abs() <= EPS);
        let data: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        assert!((calc(&data) - 4.5).abs() <= EPS);
        let data: Vec<f64> = vec![3.0, 5.0, 4.0, 8.0, 1.0, 7.0, 2.0, 6.0];
        assert!((calc(&data) - 4.5).abs() <= EPS);
        let data: Vec<f64> = vec![4.0, 6.0, 3.0, 8.0, 1.0, 7.0, 2.0, 5.0];
        assert!((calc(&data) - 4.5).abs() <= EPS);
        let data: Vec<f64> = vec![5.0, 6.0, 3.0, 8.0, 1.0, 7.0, 2.0, 4.0];
        assert!((calc(&data) - 4.5).abs() <= EPS);
        let data: Vec<f64> = vec![1.0, 2.0, 2.0, 3.0, 3.0, 3.0, 3.0, 4.0];
        assert!((calc(&data) - 3.0).abs() <= EPS);
        let data: Vec<f64> = vec![1.0, 2.0, 2.0, 2.0, 3.0, 3.0, 3.0, 3.0, 4.0];
        assert!((calc(&data) - 3.0).abs() <= EPS);
    }

    #[test]
    fn moremedian() {
        let data: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        assert!((calc(&data) - 4.5).abs() <= EPS);
        let data: Vec<f64> = vec![1.0, 2.0, 2.0, 3.0, 3.0, 3.0, 3.0, 4.0];
        assert!((calc(&data) - 3.0).abs() <= EPS);

        let data: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        assert!((calc(&data) - 4.5).abs() <= EPS);
        let data: Vec<f64> = vec![3.0, 5.0, 4.0, 8.0, 1.0, 7.0, 2.0, 6.0];
        assert!((calc(&data) - 4.5).abs() <= EPS);
        let data: Vec<f64> = vec![1.0, 2.0, 2.0, 3.0, 3.0, 3.0, 3.0, 4.0];
        assert!((calc(&data) - 3.0).abs() <= EPS);

        let data: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        assert!((calc(&data) - 4.0).abs() <= EPS);
        let data: Vec<f64> = vec![1.0, 1.0, 1.0, 4.0, 5.0, 6.0, 1.0];
        assert!((calc(&data) - 1.0).abs() <= EPS);
        let data: Vec<f64> = vec![1.0, 1.0, 2.0, 4.0, 5.0, 6.0, 1.0];
        assert!((calc(&data) - 2.0).abs() <= EPS);
        let data: Vec<f64> = vec![4.0, 2.0, 1.0, 7.0, 3.0, 6.0, 5.0];
        assert!((calc(&data) - 4.0).abs() <= EPS);
        let data: Vec<f64> = vec![7.0, 7.0, 1.0, 1.0, 5.0, 4.0, 3.0];
        assert!((calc(&data) - 4.0).abs() <= EPS);
        let data: Vec<f64> = vec![5.0, 3.0, 4.0, 7.0, 1.0, 6.0, 2.0];
        assert!((calc(&data) - 4.0).abs() <= EPS);

        let data: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        assert!((calc(&data) - 4.0).abs() <= EPS);
        let data: Vec<f64> = vec![1.0, 1.0, 1.0, 4.0, 5.0, 6.0, 1.0];
        assert!((calc(&data) - 1.0).abs() <= EPS);
        let data: Vec<f64> = vec![1.0, 1.0, 2.0, 4.0, 5.0, 6.0, 1.0];
        assert!((calc(&data) - 2.0).abs() <= EPS);
        let data: Vec<f64> = vec![4.0, 2.0, 1.0, 7.0, 3.0, 6.0, 5.0];
        assert!((calc(&data) - 4.0).abs() <= EPS);
        let data: Vec<f64> = vec![7.0, 7.0, 1.0, 1.0, 5.0, 4.0, 3.0];
        assert!((calc(&data) - 4.0).abs() <= EPS);
        let data: Vec<f64> = vec![5.0, 3.0, 4.0, 7.0, 1.0, 6.0, 2.0];
        assert!((calc(&data) - 4.0).abs() <= EPS);

        let data: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 4.0, 5.0, 6.0, 7.0];
        assert!((calc(&data) - 4.0).abs() <= EPS);

        let data: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 6.0, 20.0];
        assert!((calc(&data) - 4.5).abs() <= EPS);
        let data: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 5.0, 6.0, 6.0];
        assert!((calc(&data) - 4.5).abs() <= EPS);

        let data: Vec<f64> = vec![1.0, 2.0, 3.0, 5.0];
        assert!((calc(&data) - 2.5).abs() <= EPS);

        let data: Vec<f64> = vec![1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 256.0];
        assert!((calc(&data) - 16.0).abs() <= EPS);
    }
}
