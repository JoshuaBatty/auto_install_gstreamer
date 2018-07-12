use std::io::{self, BufRead};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;

fn main() {
    install();
}

//------------------INSTALL HOMEBREW & GSTREAMER
fn install() {
    // Check to see if Homebrew is installed, and install if its not
    if check_for_homebrew_installation() {
        println!("Installing Homebrew");
        run_command(install_hombrew().unwrap()).unwrap();
    } else {
        println!("Existing Homebrew installation detected");
    }

    // Once we have homebrew on the system check if gstreamer is already installed
    if check_for_gstreamer_installation() {
        println!("Package gstreamer is not installed, preparing to download and install");
        run_command(install_gstreamer().unwrap()).unwrap();
    } else {
        println!("Existing gstreamer package is already installed");
    }
}

//------------------UNINSTALL HOMEBREW & GSTREAMER
fn uninstall() {
    // Check to see if Homebrew is installed, and install if its not
    if check_for_homebrew_installation() {
        println!("There is already no Homebrew installation!");
    } else {
        println!("Uninstallting Homebrew");
        run_command(uninstall_hombrew().unwrap()).unwrap();
    }

    // Once we have homebrew on the system check if gstreamer is already installed
    if check_for_gstreamer_installation() {
        println!("There is already no gstreamer formula installed!");
    } else {
        println!("Uninstallting Gstreamer");
        run_command(uninstall_gstreamer().unwrap()).unwrap();
    }
}

fn _search_gstreamer() -> io::Result<std::process::Child> {
    let mut child = Command::new("brew");
    let child = child
        .arg("search")
        .arg("gstreamer")
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn();

    child
}

fn check_for_homebrew_installation() -> bool {
    let output = Command::new("sh")
        .arg("-c")
        .arg("command -v brew")
        .output()
        .expect("failed to execture process");
    String::from_utf8_lossy(&output.stdout).is_empty()
}

fn install_hombrew() -> io::Result<std::process::Child> {
    let mut child = Command::new("/usr/bin/ruby");
    let child = child
        .arg("-e")
        .arg("$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install)")
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn();

    child
}

fn uninstall_hombrew() -> io::Result<std::process::Child> {
    let mut child = Command::new("ruby");
    let child = child
        .arg("-e")
        .arg("$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/uninstall)")
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn();

    child
}

fn check_for_gstreamer_installation() -> bool {
    let output = Command::new("brew")
        .arg("ls")
        .arg("--versions")
        .arg("gstreamer")
        .output()
        .expect("failed to execture process");
    String::from_utf8_lossy(&output.stdout).is_empty()
}

fn install_gstreamer() -> io::Result<std::process::Child> {
    let mut child = Command::new("brew");
    let child = child
        .arg("install")
        .arg("gstreamer")
        .arg("gst-plugins-base")
        .arg("gst-plugins-good")
        .arg("gst-plugins-bad")
        .arg("gst-plugins-ugly")
        .arg("gst-libav")
        .arg("gst-rtsp-server")
        .arg("--with-orc")
        .arg("-with-libogg")
        .arg("--with-opus")
        .arg("--with-pango")
        .arg("--with-theora")
        .arg("--with-libvorbis")
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn();

    child
}

fn uninstall_gstreamer() -> io::Result<std::process::Child> {
    let mut child = Command::new("brew");
    let child = child
        .arg("uninstall")
        .arg("gstreamer")
        .arg("gst-plugins-base")
        .arg("gst-plugins-good")
        .arg("gst-plugins-bad")
        .arg("gst-plugins-ugly")
        .arg("gst-libav")
        .arg("gst-rtsp-server")
        .arg("--force")
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn();

    child
}

fn _echo(string: &'static str) {
    let output = Command::new("sh")
        .arg("-c")
        .arg("echo ".to_owned() + &string)
        .output()
        .expect("failed to execute echo process");
    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
}

fn run_command(mut child: std::process::Child) -> io::Result<()> {
    enum Line {
        Stdout(String),
        Stderr(String),
    }

    // Send lines from stdout and stderr threads to this thread.
    let (line_tx, line_rx) = mpsc::channel();

    // Spawn a thread that reads stdout from the child.
    let mut child_stdout =
        io::BufReader::new(child.stdout.take().expect("no stdout on child process"));
    let stdout_tx = line_tx.clone();
    let _stdout_thread = thread::Builder::new()
        .name("stdout_reader".into())
        .stack_size(10_000) // 10kb
        .spawn(move || {
            let mut line = String::new();
            loop {
                line.clear();
                let result = child_stdout.read_line(&mut line);
                if result.is_err() || line.is_empty() {
                    break;
                }
                let line = line.trim();
                if stdout_tx.send(Line::Stdout(line.to_string())).is_err() {
                    break;
                }
            }
        })
        .unwrap();

    // Spawn a thread that reads stderr from the child.
    let mut child_stderr =
        io::BufReader::new(child.stderr.take().expect("no stderr on child process"));
    let stderr_tx = line_tx;
    let _stderr_thread = thread::Builder::new()
        .name("stderr_reader".into())
        .stack_size(10_000) // 10kb
        .spawn(move || {
            let mut line = String::new();
            loop {
                line.clear();
                let result = child_stderr.read_line(&mut line);
                if result.is_err() || line.is_empty() {
                    break;
                }
                let line = line.trim();
                if stderr_tx.send(Line::Stderr(line.to_string())).is_err() {
                    break;
                }
            }
        }).unwrap();

    for line in line_rx {
        match line {
            Line::Stdout(line) => println!("{}", line),
            Line::Stderr(line) => eprintln!("{}", line),
        }
    }

    child.kill()?;
    Ok(())
}