/*!
 * Solve problem using manually written simulation working in main thread.  
 * Version 2: Points evaluated from highest ground to lowest.
 */

use std::iter::FromIterator;

use anyhow::Result;

#[cfg(feature = "state_fun_bd")]
#[allow(unused_imports)]
use bigdecimal::{BigDecimal, Zero};

// use crate::PointHeight;
/// Base unclehood type used for calculations during simulation in this module.
type PointHeight = <Landscape as crate::Solver>::PointHeight;

/// If water level is less than this value water does not flow from point to point.
/// Note: Placing 0.0 here may cause program to fall into infinite loop because of rounding errors.
const VISCOSITY_COEF: PointHeight = 0.01;

// TODO: Point1D could be made a template parameter, with some generic trait implementation.
// The same algorithm may work for other topologies (e.g. Point2D, generic Point with its own list of neighbors, etc.`)
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
        Landscape { points, points_idx, results:ph, precision:VISCOSITY_COEF }
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
                let pw = self.points[*pi].water;
                if pw <= self.precision {
                    continue;
                }
                send_water_to.clear(); 
                let ph = self.points[*pi].get_height();
                for ni in self.neighbors(*pi) {
                    let nh = self.points[ni].get_height();
                    if ph > nh + self.precision {
                        send_water_to.push(ni);
                    }
                }
                if send_water_to.is_empty() {
                    continue;
                }
                let equal_fraction = pw / send_water_to.len() as PointHeight;
                for ni in &send_water_to {
                    let diff = self.points[*pi].get_height() - self.points[*ni].get_height();
                    if diff > self.precision {
                        let flow_amt = if equal_fraction < diff / 2.0 { equal_fraction } else { diff / 2.0 };
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
                self.points[wu.from_idx].water -= wu.water;
                self.points[wu.to_idx].water += wu.water;
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

impl crate::Solver for Landscape {
    /// Base unclehood type used for calculations during simulation in this module.
    type PointHeight = f64; 
    
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
    fn precision(&self) -> PointHeight { self.precision }
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
            water: 0.0,
        }
    }

    /// Returns level of water (dry point height + water over it)
    #[inline]
    fn get_height(&self) -> PointHeight {
        self.ground + self.water
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

    include!("test_common_f64.inc.rs");
}
