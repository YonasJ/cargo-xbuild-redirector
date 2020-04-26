use std::env;
use std::path::{PathBuf, Path};
use std::process::{exit, Command};
use clap::{Arg, App, SubCommand};
use ex::fs;
use regex::Regex;
use ex::fs::copy;

struct CargoRedirectHelper {
    my_path: PathBuf,
    my_host_triple: String
}

const CRATE_NAME:&str = "cargo-xbuild-redirector";
const REDIRECTED_CARGO_NAME:&str = "cargo-xbuild-redirector-real";


impl CargoRedirectHelper {
    fn new() -> Result<CargoRedirectHelper, std::io::Error> {
        // let me = env::current_exe()?;
        // let me_ext = (&me).extension().unwrap_or(OsStr::new("")).to_str().unwrap();

        Ok(CargoRedirectHelper {
            my_path: env::current_exe()?,
            my_host_triple: platforms::guess_current().expect("Unable to determine platform").target_triple.to_owned()
        })
    }

    fn install(&self) -> Result<(), std::io::Error> {
        let matches = App::new(CRATE_NAME)
            .version("0.1.0")
            .author("Yonas Jongkind@gmail.com")
            .about(format!("If the cargo target is build, this checks the ${{PWD}}/.cargo/config file for 'target=xxxx'. If it finds that line, it runs {} xbuild. Run with install --toolchain <toolchain_name> to install.", REDIRECTED_CARGO_NAME).as_str())
            .subcommand(SubCommand::with_name("install")
                .about("Installs the binary into a rustup tool chain.")
                .arg(Arg::with_name("toolchain")
                    .long("toolchain")
                    .short("-T")
                    .number_of_values(1)
                    .takes_value(true)
                    .help("Name of the rustup toolchain to install into"))
            ).get_matches();

        if let Some(install) = matches.subcommand_matches("install") {
            let mut cmd = Command::new("rustup");

            if let Some(toolchain) = install.value_of("toolchain") {
                println!("Installing for toolchain {}", toolchain);
                cmd.arg(format!("+{}",toolchain));
            }
            cmd.arg("which");
            cmd.arg("cargo");
            match cmd.output() {
                Ok(output) => {
                    if output.status.success() && output.status.code().unwrap() == 0 {
                        // println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
                        let cargo_path_str = format!("{}", String::from_utf8_lossy(&output.stdout).trim());
                        let cargo_base = PathBuf::from(&cargo_path_str);
                        let mut cargo_redir = PathBuf::from(&cargo_path_str);
                        cargo_redir.set_file_name(REDIRECTED_CARGO_NAME);

                        if !cargo_base.exists() && !cargo_redir.exists() {
                            println!("Install aborted - cannot find cargo at expected location: {:?}", cargo_base);
                            exit(1);
                        }

                        if !cargo_redir.exists() {
                            println!("Moving {:?} to {:?} to install re-director.", cargo_base, cargo_redir);
                            copy(&cargo_base, &cargo_redir).unwrap();
                        }
                        println!("Moving {:?} to {:?} to install re-director.", self.my_path, cargo_base);
                        copy(&self.my_path, &cargo_base).unwrap();

                        return Ok(());
                    } else {
                        println!("Failed to execute rustup: {}\n{}{}", output.status, String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
                    }
                }
                Err(e) => {
                    println!("Unable to execute `rustup which cargo` to determine version: {:?}",e);
                    exit(1);
                }
            }
        } else {
            matches.usage();
        }

        Ok(())
    }


    fn get_cargo_config_build_target(&self) -> Result<String, std::io::Error> {
        let cargo_config_path = Path::new(".cargo/config");
        if !cargo_config_path.exists() {
            println!("No .cargo/config file exists.");
            return Ok(self.my_host_triple.clone());
        }
        let cargo_config = fs::read_to_string(cargo_config_path).unwrap();

        let re = Regex::new(r#"\s+target\s*=\s*"(.*)""#).unwrap();
        let caps = re.captures(&cargo_config);
        if caps.is_some() {
            let mgv = caps.unwrap().get(1);
            if mgv.is_some() {
                let target_host_triple = String::from(mgv.unwrap().as_str());
                // println!("matched target = {}", target_host_triple);
                return Ok(target_host_triple);
            } else {
                // println!("No matching group");
            }
        } else {
            // println!("No 'target =' line in .cargo/config");
        }
        Ok(self.my_host_triple.clone())
    }

    fn run_cargo(&self) -> Result<(), std::io::Error> {
        match self.get_cargo_config_build_target() {
            Ok(target) => {
                if target.eq(&self.my_host_triple) {
                    println!("{} redirecting to real cargo for target={:?}", CRATE_NAME, target);
                    self.run_real_cargo(false);
                } else {
                    println!("{} redirecting to real cargo with xbuild because target={:?} does not match host {:?}", CRATE_NAME, target, self.my_host_triple);
                    self.run_real_cargo(true);
                }
            },
            Err(e) => {
                println!("{} unable to determine target: {:?}", CRATE_NAME, e);
            }
        }

        Ok(())
    }

    fn run_real_cargo(&self, as_xbuild:bool) {
        let mut redir_path = PathBuf::from(self.my_path.as_os_str());
        redir_path.set_file_name(REDIRECTED_CARGO_NAME);

        // let mut cmd = Command::new("echo");
        let mut cmd = Command::new(redir_path);
        let mut firstSeen = false;
        for arg in env::args() {
            if !firstSeen {
                firstSeen = true;
                continue;
            }
            if as_xbuild && arg.eq("build") {
                cmd.arg("xbuild");
            } else {
                cmd.arg(arg);
            }
        }
        exit(cmd.status().unwrap().code().unwrap());
    }
}

fn main_helper() -> Result<(), std::io::Error> {
    let r = CargoRedirectHelper::new().unwrap();

    let am_cargo = r.my_path.file_stem().unwrap().eq("cargo");
    if am_cargo {
        r.run_cargo()?;
    } else {
        r.install()?;
    }
    Ok(())
}

fn main() {
    match main_helper() {
        Ok(_) => exit(0),
        Err(e) => {
            println!("Failed: {:?}", e);
            exit(1);
        }
    }
}
