/*!
    Program to solve the problem of water flow in landscape and calculate water level after some amount of rain.
    Details in the file `Rust Programming Test.pdf`.
 ```
Usage:
   sv_challenge N [<input.txt] [>output.txt]
where:
   N          - finish after this number of rain simulations (hours in the task description)
   input.txt  - text file with landscape definition: one landscape point with float hight in one line
   output.txt - results - at every simulation step (hour) a line is printed with comma separated water hights per point in input file order 
```
   Please look at `README.md` for more information.  
   License: MIT like - see `LICENSE`.  
   Copyright (c) 2020 Grzegorz Wierzchowski.
 */

#[macro_use] extern crate anyhow;

use std::env;
use std::io::{stdin, stdout, Write};

use anyhow::Result;

/// Amount of rain that falls onto one point (segment) in one step (1h).
const RAIN_DENSITY: f64 = 1.0;

mod simul_manual_1th_v1;
mod simul_manual_1th_v2;
#[cfg(feature = "bigdecimal")]
mod simul_manual_1th_bd_v2;
// mod simul_manual_1th_v3;


/// Creates concrete object used to solve problem.
fn solver_factory(points_heights: Vec<f64>) -> impl Solver {
    // simul_manual_1th_v1::Landscape::create(points_heights)
    simul_manual_1th_v2::Landscape::create(points_heights)
}

/// Program main function.
fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect(); // TODO: use clap here
    if args.len() < 2 {
        bail!(
r#"Missing command line parameter

Usage:
   {} N [<input.txt] [>output.txt]
where:
   N          - finish after this number of rain simulations (hours in the task description)
   input.txt  - text file with landscape definition: one landscape point with float hight in one line
   output.txt - results - at every simulation step (hour) a line is printed with comma separated water hights per point in input file order 
"#, args[0]);
    }

    let steps = usize::from_str_radix(&args[1], 10)?;
    let mut points = Vec::new();

    let stdin = stdin();
    let mut buf = String::new();
    loop {
        match stdin.read_line(&mut buf) {
            Ok(n) if n > 1 => {
                match buf.trim().parse::<f64>() {
                    Ok(p) => {
                        // TODO: Maybe negative is ok - check after algorithm is ready
                        // Note: Algorithm should be ok, but negative numbers may mess-up calc_state()
                        if p < 0.0 { bail!("Input line number {}: negative height: {} not allowed", points.len() + 1, p); }
                        points.push(p);
                    },
                    Err(e) => bail!("Error at input line number {}: {}", points.len() + 1, e),
                }
            },
            Ok(_) => break,
            Err(e) => bail!("Error at input line number {}: {}", points.len() + 1, e),
        }
        buf.clear();
    }
    
    let mut stdout = stdout();
    let mut landscape = solver_factory(points);
    for n in 1..=steps {
        match landscape.rain_uniform(RAIN_DENSITY.into(), true) {
            Ok(water_levels) => {
                // TODO: it should be rather format!("{}", ...
                // TODO: this may not be good, it should be reworked using IntoStr
                stdout.write_all(format!("{:?}", water_levels).trim_matches(&['[',']'] as &[_]).as_bytes())?; 
                stdout.write(&[b'\n'])?;
            },
            Err(e) => { bail!("Error during {} st/th invocation of rain(): {}", n, e); }
        }
    }
    Ok(())
}

/// Functions required to solve problem.
pub trait Solver {
    /// Type that represents point height and water height.
    /// Base unclehood type used for calculations during simulation.
    type PointHeight: std::fmt::Debug + From<f64> + Clone; //TODO: It should be rather Display

    /// Simulates one step (1h in problem description) of falling rain.  
    /// `rain_distr` - function which determines rain density (amount of water) depending on point index.  
    /// `return_result` - weather function should return result (water levels) or just simulate rain (empty slice is returned)
    fn rain(&mut self, rain_distr: impl Fn(usize) -> Self::PointHeight, return_result: bool) -> Result<&[Self::PointHeight]>;
    
    /// Default implementation in case when rain is uniform thru entire landscape (as in problem description = 1.0)
    fn rain_uniform(&mut self, cnt: Self::PointHeight, return_result: bool) -> Result<&[Self::PointHeight]> {
        self.rain(|_| cnt.clone(), return_result)
    }

    /// Returns simulation precision.
    /// If water levels difference is less than returned value, water will not flow (0 for exact simulation)
    fn precision(&self) -> Self::PointHeight;
}

