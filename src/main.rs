#[cfg(not(windows))]
compile_error!(
    "btcodec-inspector only supports Windows. Build with --target x86_64-pc-windows-msvc."
);

#[cfg(windows)]
mod codec;

#[cfg(windows)]
fn main() {
    use ferrisetw::parser::Parser;
    use ferrisetw::provider::Provider;
    use ferrisetw::schema_locator::SchemaLocator;
    use ferrisetw::trace::UserTrace;
    use ferrisetw::EventRecord;

    use crate::codec::A2dpCodec;

    // BthA2DP ETW Provider — emits A2dpStreaming events.
    const PROVIDER_GUID: &str = "8776ad1e-5022-4451-a566-f47e708b9075";
    const SESSION_NAME: &str = "BthA2DpInspectorSession";
    const STREAMING_TASK: &str = "A2dpStreaming";

    if !is_elevated::is_elevated() {
        eprintln!("ERROR: This program requires elevated privilege.");
        std::process::exit(1);
    }

    println!("Note: this utility may have delay depending on the Bluetooth chipset used, A2DP streaming event may not be presented in real time.");
    println!("Format information may appear during audio playback session or at the end of the audio playback session.");
    println!("This is a Windows Bluetooth Audio Stack limitation.");
    println!();

    let provider = Provider::by_guid(PROVIDER_GUID)
        .add_callback(|record: &EventRecord, schema_locator: &SchemaLocator| {
            let Ok(schema) = schema_locator.event_schema(record) else {
                return;
            };
            if schema.task_name() != STREAMING_TASK {
                return;
            }
            let parser = Parser::create(record, &schema);
            let standard_id: u8 = parser.try_parse("A2dpStandardCodecId").unwrap_or(0xFF);
            let vendor_id: u32 = parser.try_parse("A2dpVendorId").unwrap_or(0);
            let vendor_codec_id: u32 = parser.try_parse("A2dpVendorCodecId").unwrap_or(0);
            let codec = A2dpCodec::new(standard_id, vendor_id as i32, vendor_codec_id as i32);
            println!("A2DP Streaming event. Codec: {}", codec);
        })
        .build();

    let trace = match UserTrace::new()
        .named(SESSION_NAME.to_string())
        .enable(provider)
        .start_and_process()
    {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to start ETW session: {:?}", e);
            std::process::exit(1);
        }
    };

    let (tx, rx) = std::sync::mpsc::channel::<()>();
    ctrlc::set_handler(move || {
        let _ = tx.send(());
    })
    .expect("Failed to install Ctrl-C handler");

    let _ = rx.recv();
    let _ = trace.stop();
}
