/*!
 * Solve problem using manually written simulation working in main thread.  
 * Version 2: Points evaluated from highest ground to lowest, performs calculation on BigDecimal.
 */

use std::iter::FromIterator;

use anyhow::Result;

use bigdecimal::{BigDecimal, Zero};

// use crate::PointHeight;
/// Base unclehood type used for calculations during simulation in this module.
type PointHeight = <Landscape as crate::Solver>::PointHeight;

/// Represents entire 'world' where water is raining onto and flowing down from point (section in the paper) to point.
pub struct Landscape {
    points: Vec<Point>,
    points_idx: Vec<usize>,
    results: Vec<PointHeight>,
    precision: PointHeight,
}

#[derive(Debug)]
struct WaterUpdate {
    from_idx: usize,
    to_idx: usize,
    water: PointHeight,

    #[cfg(any(feature = "state_fun_f64", feature = "state_fun_bd"))]
    from: Point,
    #[cfg(any(feature = "state_fun_f64", feature = "state_fun_bd"))]
    to: Point,
}

impl Landscape {
    /// Create Landscape object.
    /// `points` object is intentionally consumed to free memory as soon as possible.
    /// In this case however it is moved and re-used as buffer for results.
    #[allow(dead_code)]
    pub fn create(ph: Vec<f64>) -> Self {
        let mut points = Vec::with_capacity(ph.len());
        for h in &ph {
            points.push(Point::with_height((*h).into()));
        }
        let mut points_idx = Vec::from_iter((0..ph.len()).into_iter());
        points_idx.sort_unstable_by(|i, j| ph[*j].partial_cmp(&ph[*i]).unwrap());
        let results = Vec::from_iter(ph.into_iter().map(|h| h.into()));
        Landscape { points, points_idx, results, precision:BigDecimal::from(0.01) }
    }

    /// Create Landscape object.
    /// `points` object is intentionally consumed to free memory as soon as possible.
    /// `precision` precision in which to perform simulation, the less the worse performance.
    /// Warning: setting precision equal to zero may cause simulation hang.
    #[allow(dead_code)]
    pub fn create_with_precision(ph: Vec<f64>, precision: PointHeight) -> Self {
        let mut landscape = Self::create(ph);
        landscape.precision = precision;
        landscape
    }

    // TODO: This could be implemented as different specializations for different points passed as template parameter
    /// Determines directions in which water can flow from point at `idx` index.
    fn neighbors(&self, idx: usize) -> impl Iterator<Item=usize> {
        Iter1D {idx, max:self.points.len(), iter:0}
    }

    /// Function that determines how water is flowing thru landscape.
    /// Please look at `README.md` for more information.
    fn stabilize_water(&mut self) -> Result<()> {
        #[cfg(any(feature = "state_fun_f64", feature = "state_fun_bd"))]
        let (state_lbound, mut state) = (self.calc_state_lbound(), self.calc_state());

        let mut send_water_to = Vec::new(); // TODO: possibly use smallvec or tiny_vec
        let mut water_update = Vec::new();
        loop {
            water_update.clear();
            for pi in &self.points_idx {
                let pw = self.points[*pi].water.clone();
                if pw <= self.precision {
                    continue;
                }
                send_water_to.clear(); 
                let ph = self.points[*pi].get_height();
                for ni in self.neighbors(*pi) {
                    let nh = self.points[ni].get_height();
                    if ph > nh + &self.precision {
                        send_water_to.push(ni);
                    }
                }
                if send_water_to.is_empty() {
                    continue;
                }
                let equal_fraction = pw / BigDecimal::from(send_water_to.len() as f64);
                for ni in &send_water_to {
                    let diff = self.points[*pi].get_height() - self.points[*ni].get_height();
                    if diff > self.precision {
                        let flow_amt = if equal_fraction < diff.half() { equal_fraction.clone() } else { diff.half() };
                        water_update.push(
                            WaterUpdate {
                                from_idx: *pi,
                                to_idx: *ni,
                                water: flow_amt,
                                
                                #[cfg(any(feature = "state_fun_f64", feature = "state_fun_bd"))]
                                from: self.points[*pi].clone(),
                                #[cfg(any(feature = "state_fun_f64", feature = "state_fun_bd"))]
                                to: self.points[*ni].clone(),
                            }
                        );
                    }
                }
            }
            if water_update.is_empty() {
                break;
            }
            for wu in &mut water_update {
                self.points[wu.from_idx].water -= wu.water.clone();
                self.points[wu.to_idx].water += wu.water.clone();
            }
            
            #[cfg(any(feature = "state_fun_f64", feature = "state_fun_bd"))] {
            let new_state = self.calc_state();
                if state < state_lbound {
                    dbg!(&water_update);
                    bail!("State function check failed: state ({}) < low bound ({})", state, state_lbound);
                }
                if new_state < state_lbound {
                    dbg!(&water_update);
                    bail!("State function check failed: new_state ({}) < low bound ({})", new_state, state_lbound);
                }
                if new_state > state {
                    dbg!(&water_update);
                    bail!("State function check failed: new_state ({}) > prev_state ({})", new_state, state);
                }
                if new_state == state {
                    dbg!(&water_update);
                    bail!("State function check failed: new_state ({}) == prev_state ({}); Function should return before", new_state, state);
                }
                // dbg!(&state_lbound, &new_state, &state);
                state = new_state;
            }
        }
        Ok(())
    }
    
    #[cfg(feature = "state_fun_f64")]
    fn calc_state(&self) -> f64 {
        let mut state = 0.0;
        for p in &self.points {
            state += (p.get_height() as f64).powf(1.4);
        }
        state        
    }

    #[cfg(feature = "state_fun_f64")]
    fn calc_state_lbound(&self) -> f64 {
        let mut lbound = 0.0;
        for p in &self.points {
            lbound += (p.ground as f64).powf(1.4);
        }
        lbound        
    }

    /* Note: I created those versions while debugging failure on calc_state checks.
       But it turned out to be caused by PointHeight being f32 instead of f64.
       In practice this versions seems to be useless because of huge performance degradation in compare to f64.
       But left them in place just in case.
     */
    #[cfg(all(feature = "state_fun_bd", not(feature = "state_fun_f64")))]
    fn calc_state(&self) -> BigDecimal {
        let mut state = Zero::zero();
        for p in &self.points {
            state += BigDecimal::from(p.get_height()).square();
        }
        state        
    }

    #[cfg(all(feature = "state_fun_bd", not(feature = "state_fun_f64")))]
    fn calc_state_lbound(&self) -> BigDecimal {
        let mut lbound = Zero::zero();
        for p in &self.points {
            lbound += BigDecimal::from(p.ground).square();
        }
        lbound        
    }
}
// impl Copy for BigDecimal {

// }

impl crate::Solver for Landscape {
    /// Base unclehood type used for calculations during simulation in this module.
    type PointHeight = BigDecimal; 
    
    /// Simulates one step of falling rain.
    fn rain(&mut self, rain_distr: impl Fn(usize) -> PointHeight, return_result: bool) -> Result<&[PointHeight]> {
        for (idx, p) in self.points.iter_mut().enumerate() {
            p.rain(rain_distr(idx));
        }

        self.stabilize_water()?;

        if return_result {
            for (i, p) in self.points.iter().enumerate() {
                self.results[i] = p.get_height();
            }
            Ok(&self.results[..])
        } else {
            Ok(&[])
        }
    }
    
    /// Returns simulation precision.
    fn precision(&self) -> PointHeight { self.precision.clone() }
}

/// Represents point (section) on landscape
#[derive(Debug, Clone)]
struct Point {
    ground: PointHeight,
    water: PointHeight,
}

impl Point {
    /// Point constructor
    #[allow(dead_code)]
    fn with_height(h: PointHeight) -> Self {
        Point { 
            ground: h,
            water: Zero::zero(),
        }
    }

    /// Returns level of water (dry point height + water over it)
    #[inline]
    fn get_height(&self) -> PointHeight {
        self.ground.clone() + &self.water
    }
    
    /// Simulate `cnt` amount of water raining on point
    #[inline]
    fn rain(&mut self, cnt: PointHeight) {
        self.water += cnt;
    }
}

struct Iter1D {
    idx: usize,
    max: usize,
    iter: u8,
}

impl Iterator for Iter1D {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter {
            0 => {
                self.iter = 1;
                if self.idx > 0 { Some(self.idx - 1) } else { if self.max > 1 { Some(1) } else { None }}
            },
            1 => {
                self.iter = 2;
                if self.idx == 0 { None } else { if self.idx < self.max - 1 { Some(self.idx + 1) } else { None }}
            },
            _ => None
        }
    }
}

/* 
///Potentially to build more complicated topology
#[derive(Debug, Clone)]
struct PointFreehand {
    ground: PointHeight,
    neighbors: Vec<usize>,
}

impl PointFreehand {
    fn with_height(h: PointHeight) -> Self {
        PointFreehand { 
            ground: h,
            neighbors: Vec::with_capacity(2) 
        }
    }
    
    #[inline]
    fn add_neighbor(&mut self, other: usize) {
        self.neighbors.push(other);
    }
}
*/

///////////////////////////////////////////////////////////////////////////////////////////////////
/// Tests
/// 

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::*;
    const STRICT_EQUALITY:bool = false;

    fn compare(precision:&PointHeight, left: &[PointHeight], right: &[PointHeight]) {
        assert_eq!(left.len(), right.len());
        if precision.partial_cmp(&Zero::zero()) == Some(std::cmp::Ordering::Equal) || STRICT_EQUALITY {
            assert_eq!(left, right);
        } else {
            // let mut diff = Zero::zero();
            for (i, _r) in left.iter().enumerate() {
                if (left[i].clone() - &right[i]).abs() > precision.clone() * BigDecimal::from(left.len() as f64) {
                    assert!(false, "left[{}]={} != right[{}]={}", i, left[i], i, right[i]);
                }
                // if (left[i] - right[i]).abs() > prec {
                //     assert!(false, "left[{}]={} != right[{}]={}", i, left[i], i, right[i]);
                // }
                // diff += (left[i] - right[i]).abs();
                // if diff > prec * (left.len() as PointHeight) {
                //     assert!(false, "left={:?} != right={:?}; diff={}", left, right, diff);
                // }
            }
        }
    }

    #[test]
    fn sv_case_sample() {
        let points = vec![3.0, 1.0, 6.0, 4.0, 8.0, 9.0];
        let mut landscape = Landscape::create(points);
        let prec = landscape.precision();
        let result = landscape.rain_uniform(RAIN_DENSITY.into(), true).unwrap();
        compare(&prec, result, [4.0, 4.0, 6.0, 6.0, 8.0, 9.0]
            .iter().map(|h| BigDecimal::from(*h)).collect::<Vec<BigDecimal>>().as_slice()
        );
    }

    #[test]
    fn sv_case_mail2() {
        let points = vec![8.0, 8.0, 1.0];
        let mut landscape = Landscape::create(points);
        let prec = landscape.precision();
        let result = landscape.rain_uniform(RAIN_DENSITY.into(), true).unwrap();
        compare(&prec, result, [8.0, 8.0, 4.0]
            .iter().map(|h| BigDecimal::from(*h)).collect::<Vec<BigDecimal>>().as_slice()
        );
    }

    #[test]
    fn sv_case_mail3() {
        let points = vec![1.0, 8.0, 8.0, 1.0];
        let mut landscape = Landscape::create(points);
        let prec = landscape.precision();
        let result = landscape.rain_uniform(RAIN_DENSITY.into(), true).unwrap();
        compare(&prec, result, [3.0, 8.0, 8.0, 3.0]
            .iter().map(|h| BigDecimal::from(*h)).collect::<Vec<BigDecimal>>().as_slice()
        );
    }

    #[test]
    fn sv_case_mail4() {
        let points = vec![8.0, 4.0, 8.0, 8.0, 1.0];
        let mut landscape = Landscape::create(points);
        let prec = landscape.precision();
        let result = landscape.rain_uniform(RAIN_DENSITY.into(), true).unwrap();
        compare(&prec, result, [8.0, 7.0, 8.0, 8.0, 3.0]
            .iter().map(|h| BigDecimal::from(*h)).collect::<Vec<BigDecimal>>().as_slice()
        );
    }

    #[test]
    fn sv_case_mail5() {
        let points = vec![1.0, 8.0, 8.0, 8.0, 1.0];
        let mut landscape = Landscape::create(points);
        let prec = landscape.precision();
        let result = landscape.rain_uniform(RAIN_DENSITY.into(), true).unwrap();
        compare(&prec, result, [3.5, 8.0, 8.0, 8.0, 3.5]
            .iter().map(|h| BigDecimal::from(*h)).collect::<Vec<BigDecimal>>().as_slice()
        );
    }
}
