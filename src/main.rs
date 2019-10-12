use std::collections::BTreeMap;
use serde_derive::Deserialize;
use clap::App;

#[macro_use]
extern crate clap;

#[macro_use]
extern crate rust_embed;

#[derive(Debug)]
enum Error {
    NoSuchRange,
}

type Result<T> = core::result::Result<T, Error>;

fn min(a: f64, b: f64) -> f64 {
    if a < b {
        a
    } else {
        b
    }
}

fn max(a: f64, b: f64) -> f64 {
    if a > b {
        a
    } else {
        b
    }
}

#[derive(Debug)]
struct CustomTrc {
    pub gain_bandwidth: f64,
    pub compensation_freq: f64,
    pub pole_zero_ratio: f64,
}

#[derive(Deserialize, Debug)]
struct DeviceSpecification {
    loop_delay: f64,
    max_pole_zero_ratio: f64,
    offset_resistance: f64,
    ranges: BTreeMap<String, f64>,
}

impl DeviceSpecification {
    fn calculateResistance(&self, range: &str) -> Result<f64> {
        let range = match self.ranges.get(range) {
            Some(x) => x,
            None => {
                println!("{}", range);
                return Err(Error::NoSuchRange);
            }
        };
        Ok(range + self.offset_resistance)
    }

    pub fn calculate(&self, range: &str, capacitance: f64, gain_bandwidth: f64) -> Result<CustomTrc> {
        let maxpzr_sq = (self.max_pole_zero_ratio).sqrt();
        let pole_added_by_cap = 1.0 / (2.0 * 3.14159265 * ((self.calculateResistance(range)? * capacitance) + self.loop_delay));
        let provisional_zero = max(
            pole_added_by_cap,
            (gain_bandwidth * pole_added_by_cap / maxpzr_sq).sqrt()
        );
        let fp = min(
            provisional_zero * self.max_pole_zero_ratio,
            gain_bandwidth * 4.0
        );
        let fz = min(
            provisional_zero,
            fp * self.max_pole_zero_ratio
        );
        let comp_freq = (fp * fz).sqrt();
        let pzr = fp / fz;
        Ok(CustomTrc{
            gain_bandwidth: gain_bandwidth,
            compensation_freq: comp_freq,
            pole_zero_ratio: pzr
        })
    }

    pub fn printRanges(&self) {
        for key in self.ranges.keys() {
            println!("-i {}", key);
        }
    }
}

#[derive(RustEmbed)]
#[folder = "devices/"]
struct Asset;

fn print_devices() {
    for fname in Asset::iter() {
        let parts: Vec<&str> = fname.split(".").collect();
        if parts[parts.len()-1] == "toml" {
            println!("-d {}", fname.split(".").collect::<Vec<&str>>()[0]);
        }
    }
}

fn main() {
    let yml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yml).get_matches();

    let device = match matches.value_of("device") {
        Some(x) => x,
        None => {
            println!("You must specify a device using the -d flag! Possible options are:");
            print_devices();
            return;
        }
    };

    let devspec = match Asset::get(&format!("{}.toml", device)) {
        Some(x) => x,
        None => {
            println!("Could not load device {}. Possible devices:", device);
            print_devices();
            return;
        }
    };
    let devspec = std::str::from_utf8(devspec.as_ref()).unwrap();
    let devspec: DeviceSpecification = toml::from_str(devspec).unwrap();

    let range = match matches.value_of("range") {
        Some(x) => x,
        None => {
            println!("You must specify a range using the -i flag! Possible options are:");
            devspec.printRanges();
            return;
        }
    };

    let c: f64 = lexical::parse(matches.value_of("capacitance").unwrap()).expect("Could not parse capacitance argument");

    let gbw: f64 = lexical::parse(matches.value_of("bandwidth").unwrap()).expect("Could not parse gain bandwidth argument");

    let trc = devspec.calculate(range, c, gbw).unwrap();
    println!("Gain-bandwidth: {}", trc.gain_bandwidth);
    println!("Compensation frequency: {}", trc.compensation_freq);
    println!("Pole-zero ratio: {}", trc.pole_zero_ratio);
}
