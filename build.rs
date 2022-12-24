use clap_complete::generate_to;
use std::env;
use std::io::Error;

include!("src/cli.rs");

fn main() -> Result<(), Error> {
    let out = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(outdir) => outdir,
    };

    let mut cmd = build_cli();

    let path = generate_to(clap_complete::shells::Bash, &mut cmd, "imgname", &out)?;
    println!("cargo:warning=completion file generated: {:?}", path);

    let path = generate_to(clap_complete::shells::Fish, &mut cmd, "imgname", &out)?;
    println!("cargo:warning=completion file generated: {:?}", path);

    let path = generate_to(clap_complete::shells::Zsh, &mut cmd, "imgname", &out)?;
    println!("cargo:warning=completion file generated: {:?}", path);

    let path = generate_to(clap_complete::shells::PowerShell, &mut cmd, "imgname", &out)?;
    println!("cargo:warning=completion file generated: {:?}", path);

    Ok(())
}
