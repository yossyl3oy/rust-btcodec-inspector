#[cfg(not(windows))]
compile_error!(
    "btcodec-inspector binary only supports Windows. \
     Build with --target x86_64-pc-windows-msvc."
);

#[cfg(windows)]
fn main() {
    use btcodec_inspector::{watch, Error};

    println!(
        "Note: this utility may have delay depending on the Bluetooth chipset used, \
         A2DP streaming event may not be presented in real time."
    );
    println!(
        "Format information may appear during audio playback session or at the end \
         of the audio playback session."
    );
    println!("This is a Windows Bluetooth Audio Stack limitation.");
    println!();

    let watcher = match watch(|codec| {
        println!("A2DP Streaming event. Codec: {}", codec);
    }) {
        Ok(w) => w,
        Err(Error::NotElevated) => {
            eprintln!("ERROR: This program requires elevated privilege.");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Failed to start ETW session: {}", e);
            std::process::exit(1);
        }
    };

    let (tx, rx) = std::sync::mpsc::channel::<()>();
    ctrlc::set_handler(move || {
        let _ = tx.send(());
    })
    .expect("Failed to install Ctrl-C handler");
    let _ = rx.recv();
    drop(watcher);
}
