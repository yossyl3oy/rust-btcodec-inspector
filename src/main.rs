#[cfg(not(windows))]
compile_error!(
    "btcodec-inspector binary only supports Windows. \
     Build with --target x86_64-pc-windows-msvc."
);

#[cfg(windows)]
fn main() {
    use btcodec_inspector::{
        observe_default_playback, watch, CodecObservability, Error, PlaybackDevice,
    };

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

    // Inspect the default playback device first so we can warn the user
    // about cases where ETW will never see codec events (USB Audio Class
    // transmitters, irrelevant outputs, no device, …) before they sit
    // there waiting for output that won't come.
    match observe_default_playback() {
        Ok(CodecObservability::BluetoothMicrosoftStack(dev)) => {
            println!("Default playback device:");
            print_device(&dev);
            println!("  Routed via Microsoft's Bluetooth stack — codec inspection is supported.");
            println!();
        }
        Ok(CodecObservability::UsbAudioBypass(dev)) => {
            eprintln!(
                "WARNING: the default playback device is a USB Audio Class device. \
                 This is almost always a Bluetooth transmitter that handles the radio \
                 link internally — Windows only sees PCM, never the Bluetooth side, \
                 so this tool cannot observe its codec."
            );
            print_device_diagnostic(&dev);
            eprintln!(
                "         The ETW session will still start, but no events are expected \
                 unless you also have a separate Bluetooth output going through Microsoft's \
                 stack. To request implementation support for this specific device, please \
                 attach a USBView descriptor dump to a GitHub issue along with the lines above."
            );
            eprintln!();
        }
        Ok(CodecObservability::OtherOutput(dev)) => {
            println!("Default playback device:");
            print_device(&dev);
            println!(
                "  Not a Bluetooth output — this tool will only emit events if some \
                 other Bluetooth A2DP stream is also active."
            );
            println!();
        }
        Ok(CodecObservability::NoDevice) => {
            eprintln!(
                "WARNING: no default playback device is configured. The ETW session will \
                 still start, but no events are expected until a Bluetooth audio device is \
                 connected and selected."
            );
            eprintln!();
        }
        Err(e) => {
            eprintln!("WARNING: failed to inspect the default playback device: {e}");
            eprintln!();
        }
    }

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

    fn print_device(dev: &PlaybackDevice) {
        println!("  Friendly name: {}", dev.friendly_name);
        println!("  Instance ID:   {}", dev.instance_id);
        if let (Some(vid), Some(pid)) = (dev.vid, dev.pid) {
            println!("  USB VID:PID:   {:04X}:{:04X}", vid, pid);
        }
    }

    fn print_device_diagnostic(dev: &PlaybackDevice) {
        eprintln!();
        eprintln!("  Friendly name: {}", dev.friendly_name);
        eprintln!("  Instance ID:   {}", dev.instance_id);
        if let (Some(vid), Some(pid)) = (dev.vid, dev.pid) {
            eprintln!("  USB VID:PID:   {:04X}:{:04X}", vid, pid);
        }
        eprintln!();
    }
}
