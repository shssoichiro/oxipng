use std::{env, error::Error, fs, fs::File, io::BufWriter};

use clap_mangen::Man;

include!("../../src/cli.rs");

fn main() -> Result<(), Box<dyn Error>> {
    match &*env::args().nth(1).ok_or("No xtask to run provided")? {
        "mangen" => build_manpages(),
        _ => Err("Unknown xtask".into()),
    }
}

fn build_manpages() -> Result<(), Box<dyn Error>> {
    // Put manpages in <working directory>/target/xtask/mangen/manpages. Our working directory is
    // expected to be the root of the repository due to the xtask invocation alias
    let manpages_dir = env::current_dir()?.join("target/xtask/mangen/manpages");
    fs::create_dir_all(&manpages_dir)?;

    let mut man_file = BufWriter::new(File::create(manpages_dir.join("oxipng.1"))?);
    Man::new(build_command()).render(&mut man_file)?;

    println!("Manpages generated in {}", manpages_dir.display());

    Ok(())
}
