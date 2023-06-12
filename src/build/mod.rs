use pkg_config::Config;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::print;

pub fn check_database() -> Result<(), std::io::Error> {
    let mysql_presence = Config::new().probe("mariadb");
    match mysql_presence {
        Ok(v) => Ok(println!("{v:?}")),
        Err(_) => {
            if let Ok(file) = File::open("/etc/os-release") {
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    match line {
                        Ok(line) if line.starts_with("ID=") => {
                            let distribution = line[3..].to_string();
                            // Use the distribution information to install MySQL server
                            install_mysql(&distribution);
                        }
                        _ => println!("Failed to detect Linux distribution."),
                    }
                }
            }

            Ok(())
        }
    }
}

fn install_mysql(distribution: &str) {
    match distribution {
        "ubuntu" | "debian" => {
            // Use the apt package manager to install MySQL server
            println!("Installing MySQL server using apt...");
            // Execute the appropriate commands to install MySQL server using apt
        }
        "centos" | "fedora" | "rhel" => {
            // Use the yum package manager to install MySQL server
            println!("Installing MySQL server using yum...");
            // Execute the appropriate commands to install MySQL server using yum
        }
        "arch" => {
            match std::process::Command::new("sudo")
                .arg("pacman")
                .arg("-Syy")
                .arg("mariadb")
                .arg("--noconfirm")
                .spawn()
                .unwrap()
                .wait()
                .unwrap()
                .success()
            {
                true => {
                    println!("Successfully installed MySQL on arch");

                    match std::process::Command::new("sudo")
                        .arg("mariadb-install-db")
                        .arg("--user=mysql")
                        .arg("--basedir=/usr")
                        .arg("--datadir=/var/lib/mysql/")
                        .spawn()
                        .unwrap()
                        .wait()
                        .unwrap()
                        .success()
                    {
                        true => {
                            println!("Primary MariaDB has been setup.");
                            match std::process::Command::new("sudo")
                                .arg("systemctl")
                                .arg("start")
                                .arg("mysqld")
                                .spawn()
                                .unwrap()
                                .wait()
                                .unwrap()
                                .success()
                            {
                                true => {
                                    println!("MariaDB service has been started");
                                    let user_name = std::env::var("MYSQL_USERNAME").unwrap();
                                    let password = std::env::var("MYSQL_PASSWORD").unwrap();

                                    match std::process::Command::new("sudo")
                                        .arg("mariadb")
                                        .arg("-e")
                                        .arg(format!(
                                            "CREATE USER '{}'@'localhost' IDENTIFIED BY '{}'; 
                                            GRANT ALL PRIVILEGES ON *.* TO '{}'@'localhost';
                                            FLUSH PRIVILEGES;",
                                            user_name, password, user_name
                                        ))
                                        .spawn()
                                        .unwrap()
                                        .wait_with_output()
                                    {
                                        Ok(v) => println!("{:?}", v.stdout),
                                        Err(e) => panic!("{}", e.to_string()),
                                    }
                                }
                                false => panic!("Failed to execute the mysqld service"),
                            }
                        }
                        false => panic!("Error setting up MariaDB"),
                    }
                }
                false => panic!("Failed to install MySQL on arch"),
            }
        }
        _ => {
            println!("Unsupported distribution: {}", distribution);
        }
    }
}

pub fn check_redis() -> Result<(), std::io::Error> {
    let redis = Config::new().probe("redis-server");
    match redis {
        Ok(v) => Ok(println!("{v:?}")),
        Err(_) => {
            if let Ok(mut child) = std::process::Command::new("sudo")
                .arg("pacman")
                .arg("-Syy")
                .arg("redis")
                .arg("--noconfirm")
                .spawn()
            {
                match child.wait().unwrap().success() {
                    true => print!("Successfully Installed Redis"),
                    false => panic!("Redis Installation Was Unsuccessful. Aborting..."),
                }
            }
            Ok(())
        }
    }
}

fn main() -> Result<(), std::io::Error> {
    check_database()?;
    check_redis()
}
