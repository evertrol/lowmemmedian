#[macro_use]
extern crate log;
use std::cmp::Ordering;

pub fn calccounts(data: &[f64], partition: f64) -> (usize, usize, f64, f64) {
    let mut below = -f64::INFINITY;
    let mut above = f64::INFINITY;
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

pub fn calcgen(data: &[f64], maxdiff: f64, factor: f64, decrease: f64) -> f64 {
    let len = data.len();
    if len == 0 {
        return f64::NAN;
    }
    if len == 1 {
        return data[0];
    }
    if len == 2 {
        return (data[0] + data[1]) / 2.0;
    }

    let mdiff = if maxdiff >= 0.0 {
        maxdiff
    } else {
        -maxdiff * len as f64
    };

    let mut fact = factor;
    let mut prevdiff = f64::INFINITY;
    let sum: f64 = data.iter().sum();
    let mut partition = sum / (len as f64);
    let mut prevpartition = partition;
    let mut delta = 0.0;
    let evenlen = len % 2 == 0;

    debug!("Partition start (= mean), mdiff: {}, {}", partition, mdiff);

    let mut iloop: u64 = 0;
    loop {
        iloop += 1;
        let (nlow, nhigh, below, above) = calccounts(data, partition);
        // Determine break criteria for the loop, or otherwise the
        // change in partition
        let nsame = nhigh + nlow - len;
        match nlow.cmp(&nhigh) {
            Ordering::Equal => {
                if nsame == 0 {
                    partition = (below + above) / 2.0;
                }
                break;
            }
            Ordering::Greater => {
                if nlow - nhigh <= nsame {
                    partition = if nsame > 0 {
                        if evenlen && nsame == 1 {
                            (below + partition) / 2.0
                        } else {
                            partition
                        }
                    } else if evenlen {
                        (below + above) / 2.0
                    } else {
                        below
                    };
                    break;
                }
                let diff = (nlow - nhigh - nsame) as f64;
                debug!("diff, nsame = {}, {}", diff, nsame);
                if diff > mdiff {
                    if diff > prevdiff.abs() {
                        // The change was overestimated
                        // Try again with a smaller scaling factor
                        // Change `fact` by a minimum of `decrease`, but
                        // more if we overestimated the change a lot
                        let ratio = prevdiff.abs() / diff;
                        if ratio < decrease {
                            fact *= ratio;
                        } else {
                            fact *= decrease;
                        }
                        debug!(
                            "< fact, ratio, decrease = {}, {}, {}",
                            fact, ratio, decrease
                        );
                        partition = prevpartition + prevdiff * fact * delta;
                    } else {
                        prevdiff = -diff;
                        delta = above - below;
                        prevpartition = partition;
                        partition -= diff * fact * delta;
                    }
                } else {
                    partition = below;
                }
            }
            Ordering::Less => {
                if nhigh - nlow <= nsame {
                    partition = if nsame > 0 {
                        if evenlen && nsame == 1 {
                            (partition + above) / 2.0
                        } else {
                            partition
                        }
                    } else if evenlen {
                        (below + above) / 2.0
                    } else {
                        above
                    };
                    break;
                }
                let diff = (nhigh - nlow - nsame) as f64;
                debug!("diff, nsame = {}, {}", diff, nsame);
                if diff > mdiff {
                    if diff > prevdiff.abs() {
                        // The change was overestimated
                        // Try again with a smaller scaling factor
                        // Change `fact` by a minimum of `decrease`, but
                        // more if we overestimated the change a lot
                        let ratio = prevdiff.abs() / diff;
                        if ratio < decrease {
                            fact *= ratio;
                        } else {
                            fact *= decrease;
                        }
                        debug!(
                            "> fact, ratio, decrease = {}, {}, {}",
                            fact, ratio, decrease
                        );
                        partition = prevpartition + prevdiff * fact * delta;
                    } else {
                        prevdiff = diff;
                        delta = above - below;
                        prevpartition = partition;
                        partition += diff * fact * delta;
                    }
                } else {
                    partition = above;
                }
            }
        };
        debug!(
            "nlow, nhigh, below, above, delta = {}, {}, {}, {}, {}",
            nlow, nhigh, below, above, delta
        );
        debug!("iloop, partition: {}, {}", iloop, partition);
        debug!("");
    }
    debug!("iloop = {}", iloop);

    partition
}

pub fn calc(data: &[f64]) -> f64 {
    calcgen(data, 5.0, 0.2, 0.5)
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

        let data = vec![f64::INFINITY];
        let m = calc(&data);
        assert_eq!(m, f64::INFINITY);
    }

    #[test]
    fn median2() {
        let data: Vec<f64> = vec![5.0, 6.0];
        assert!((calc(&data) - 5.5).abs() <= EPS);

        let data: Vec<f64> = vec![5.0, 5.0];
        assert!((calc(&data) - 5.0).abs() <= EPS);

        let data: Vec<f64> = vec![5.0, f64::INFINITY];
        let m = calc(&data);
        assert_eq!(m, f64::INFINITY);
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

        let data: Vec<f64> = vec![5.0, 6.0, f64::INFINITY];
        assert!((calc(&data) - 6.0).abs() <= EPS);

        let data: Vec<f64> = vec![f64::INFINITY, 6.0, f64::INFINITY];
        assert_eq!(calc(&data), f64::INFINITY);
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
    }
}
