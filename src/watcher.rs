use crate::A2dpCodec;
use ferrisetw::parser::Parser;
use ferrisetw::provider::Provider;
use ferrisetw::schema_locator::SchemaLocator;
use ferrisetw::trace::{stop_trace_by_name, TraceError, UserTrace};
use ferrisetw::EventRecord;

/// `Microsoft.Windows.Bluetooth.BthA2dp` ETW provider GUID — emits
/// `A2dpStreaming` events with codec info during A2DP playback.
const PROVIDER_GUID: &str = "8776ad1e-5022-4451-a566-f47e708b9075";
const SESSION_NAME: &str = "BthA2DpInspectorSession";
const STREAMING_TASK: &str = "A2dpStreaming";

/// Errors that can occur when starting an ETW session.
#[derive(Debug)]
pub enum Error {
    /// The current process does not have administrator privilege.
    /// ETW realtime sessions require it.
    NotElevated,
    /// The underlying `ferrisetw` call failed.
    Trace(TraceError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotElevated => f.write_str("ETW realtime sessions require elevated privilege"),
            Self::Trace(e) => write!(f, "ETW trace error: {:?}", e),
        }
    }
}

impl std::error::Error for Error {}

impl From<TraceError> for Error {
    fn from(e: TraceError) -> Self {
        Self::Trace(e)
    }
}

/// RAII guard for a running ETW session. Drop to stop the session.
///
/// While this value is alive, the callback passed to [`watch`] will be
/// invoked on a background thread for every observed `A2dpStreaming` event.
/// `UserTrace`'s own `Drop` impl tears the session down, so this struct
/// just needs to keep it alive.
pub struct Watcher {
    #[allow(dead_code)]
    trace: UserTrace,
}

/// Try to extract an [`A2dpCodec`] from a single ETW event. Returns `None`
/// (and the event is silently dropped by [`watch`]) when:
///
/// - the event has no usable schema,
/// - the schema task name is not `A2dpStreaming`,
/// - `A2dpStandardCodecId` is missing or has an unexpected type, or
/// - the standard ID is `0xFF` (vendor-specific) but `A2dpVendorId` /
///   `A2dpVendorCodecId` cannot be parsed.
///
/// Skipping rather than substituting zeros keeps the CLI from emitting
/// bogus `Unknown codec (vendor 0x0000, codec 0x0000)` lines when the
/// payload format ever changes (e.g. a Windows update renames a field).
fn parse_codec(record: &EventRecord, schema_locator: &SchemaLocator) -> Option<A2dpCodec> {
    let schema = schema_locator.event_schema(record).ok()?;
    if schema.task_name() != STREAMING_TASK {
        return None;
    }
    let parser = Parser::create(record, &schema);

    // A2dpStandardCodecId is mandatory — without it we cannot identify the codec.
    let standard_id: u8 = parser.try_parse("A2dpStandardCodecId").ok()?;

    // Vendor ID and Vendor Codec ID are only meaningful (and only required)
    // for vendor-specific codecs.
    let (vendor_id, vendor_codec_id) = if standard_id == 0xFF {
        let v: u32 = parser.try_parse("A2dpVendorId").ok()?;
        let c: u32 = parser.try_parse("A2dpVendorCodecId").ok()?;
        (v as i32, c as i32)
    } else {
        (0, 0)
    };

    Some(A2dpCodec::new(standard_id, vendor_id, vendor_codec_id))
}

/// Start watching for `A2dpStreaming` ETW events from the
/// `Microsoft.Windows.Bluetooth.BthA2dp` provider. The callback is invoked
/// once per event from which a codec can be fully parsed; events with
/// missing or unexpected fields are silently skipped (see [`parse_codec`]).
/// `callback` runs on a background thread managed by `ferrisetw`.
///
/// `callback` is `FnMut`, so captured state can be mutated directly
/// (e.g. `let mut buffer = Vec::new(); watch(move |c| buffer.push(c))`).
/// The bounds match those of `ferrisetw::ProviderBuilder::add_callback`.
///
/// Requires the current process to be running with administrator
/// privileges; otherwise [`Error::NotElevated`] is returned.
///
/// The returned [`Watcher`] is an RAII guard — drop it to stop the
/// underlying ETW session.
pub fn watch<F>(mut callback: F) -> Result<Watcher, Error>
where
    F: FnMut(A2dpCodec) + Send + Sync + 'static,
{
    if !is_elevated::is_elevated() {
        return Err(Error::NotElevated);
    }

    // ETW realtime sessions live in the kernel and outlive the process that
    // created them if it didn't get to run `Drop` (e.g. crashed, killed via
    // Task Manager, debugger detach). On the next run, `start_and_process`
    // would then fail with `EvntraceNativeError::AlreadyExist`. Stop any
    // session we may have left behind under our well-known name first; the
    // error is ignored because most of the time there is nothing to stop.
    let _ = stop_trace_by_name(SESSION_NAME);

    let provider = Provider::by_guid(PROVIDER_GUID)
        .add_callback(
            move |record: &EventRecord, schema_locator: &SchemaLocator| {
                if let Some(codec) = parse_codec(record, schema_locator) {
                    callback(codec);
                }
            },
        )
        .build();

    let trace = UserTrace::new()
        .named(SESSION_NAME.to_string())
        .enable(provider)
        .start_and_process()?;

    Ok(Watcher { trace })
}
