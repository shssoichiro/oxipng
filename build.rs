use clap_mangen::Man;
use std::{env, fs::File, io::Error, path::Path};

include!("src/cli.rs");

fn build_manpages(outdir: &Path) -> Result<(), Error> {
    let app = build_command();

    let file = Path::new(&outdir).join("oxipng.1");
    let mut file = File::create(file)?;

    Man::new(app).render(&mut file)?;

    Ok(())
}

fn main() -> Result<(), Error> {
    println!("cargo:rerun-if-changed=src/cli.rs");
    println!("cargo:rerun-if-changed=man");

    let outdir = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(outdir) => outdir,
    };

    // Create `target/assets/` folder.
    let out_path = PathBuf::from(outdir);
    let mut path = out_path.ancestors().nth(4).unwrap().to_owned();
    path.push("assets");
    std::fs::create_dir_all(&path).unwrap();

    build_manpages(&path)?;

    Ok(())
}
