mod pak;
mod iohelper;

use std::io::{Read, Seek};
use clap::{Parser, ValueEnum};
use std::path::Path;
use std::process::exit;
use clap::builder::PossibleValue;
use logma::{debug, fatal, info};

// TODO:
//  - port stuff over from the python prototype
//  - implement necessary types n shit
//  - do the thing

/// todo: write a description here
#[derive(Parser, Debug)]
#[command(author, about, version, long_about = None)]
#[command(arg_required_else_help(true))]
struct Args {
    /// Input pak file or unpacked directory
    #[arg(short, long, required = true)]
    input: String,

    /// Output directory to write new pak file or to unpack into
    #[arg(short, long, required = true)]
    output: String,

    /// Operation mode
    #[arg(
        short, long,
        default_value_t,
        value_enum
    )]
    mode: PackMode,

/*    /// Game target (Only set for packing, otherwise optional)
    #[arg(
        short, long,
        default_value_t,
        value_enum
    )]
    game: Games*/
}

#[derive(clap::ValueEnum, Clone, Default, Debug)]
enum PackMode {
    Pack,
    #[default]
    Unpack
}

fn check_inputs(inpath: &Path, outpath: &Path, mode: &PackMode) {
    match mode {
        PackMode::Unpack => {
            // inpath = pak [read], outpath = folder [write]
            if !(inpath.is_file() && outpath.is_dir()) {
                fatal!("invalid inputs for mode Unpack");
                exit(1);
            }
        }
        PackMode::Pack => {
            // inpath = folder [read], outpath = pak [write]
            if !(inpath.is_dir() && outpath.is_file()) {
                fatal!("invalid inputs for mode Pack");
                exit(1);
            }
        }
    }
}


// todo: game detection
// const PEGGLE_HASH: u128 = 0xC20D99977CDA57A3C8D3734A259DF57D;
// const NIGHTS_HASH: u128 = 0xCCA1ECD425800CDFCE6C020D8411A9F3;

#[derive(Clone, Default, Debug)]
enum Games {
    #[default]
    NoneOrUndetected,
    Peggle,
    PeggleNights
}

impl ValueEnum for Games {
    fn value_variants<'a>() -> &'a [Self] {
        &[Games::NoneOrUndetected, Games::Peggle, Games::PeggleNights]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            Games::NoneOrUndetected => PossibleValue::new("none").hide(true),
            Games::Peggle => PossibleValue::new("peggle").help("Peggle"),
            Games::PeggleNights => PossibleValue::new("nights").help("Peggle Nights"),
        })
    }
}


fn impl_unpack(infile: &Path, outdir: &Path) {
    info!("Loading file \"{}\".", infile.to_str().unwrap());

    let mut pak = pak::PakFile::new();
    match pak.parse(infile) {
        Ok(_) => {
            debug!("data: {:?}", pak)
        }
        Err(err) => {
            fatal!("Pak error: {:?}", err);
            exit(1);
        }
    }
    pak.dump_files(outdir);
}


fn impl_pack(indir: &Path, outfile: &Path) {

}


fn main() {
    let args = Args::parse();
    // info!("Hello, world!");

    let inpath = Path::new(&args.input);
    let outpath = Path::new(&args.output);
    let mode = &args.mode;

    check_inputs(inpath, outpath, &mode);

    match mode {
        PackMode::Unpack => impl_unpack(inpath, outpath),
        PackMode::Pack => impl_pack(inpath, outpath),
    }
}
