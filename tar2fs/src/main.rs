use clap::{Parser, ValueEnum};
use fatfs::{FatType, FormatVolumeOptions, FsOptions};
use std::path::PathBuf;

#[derive(Debug, Copy, Clone, ValueEnum)]
enum Fs {
    Fat32,
}

#[derive(Parser)]
struct Args {
    #[arg(short, long, value_enum, default_value = "fat32")]
    r#type: Fs,
    #[arg(value_parser)]
    infile: PathBuf,
    #[arg(value_parser)]
    outfile: PathBuf,
    #[arg(short, long, default_value_t = 64*1024*1024/512)]
    sectors: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut tar =
        tar::Archive::new(std::fs::File::open(args.infile).expect("failed to open fs.tar"));
    let mut iso = std::fs::File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(args.outfile)
        .expect("failed to create fs.img");
    fatfs::format_volume(
        &mut iso,
        FormatVolumeOptions::new()
            .bytes_per_sector(512)
            .total_sectors(args.sectors)
            .fat_type(FatType::Fat32),
    )?;
    let iso = fatfs::FileSystem::new(iso, FsOptions::new())?;
    assert_eq!(iso.fat_type(), FatType::Fat32);
    for entry in tar.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        let Some(name) = path.to_str() else {
            panic!("failed to convert path to string: {path:?}")
        };

        let dir = iso.root_dir();
        dir.create_file(name)
            .expect("failed to create file in fat32");
        let mut file = dir.open_file(name)?;
        std::io::copy(&mut entry, &mut file)?;
    }
    Ok(())
}
