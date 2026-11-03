// SPDX-License-Identifier: GPL-3.0-or-later
//! Command line of EH3D-LAB-BRIDGE.

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = eh3d_lab_bridge::cli::run(&args, &mut std::io::stdout(), &mut std::io::stderr());
    std::process::exit(code);
}
