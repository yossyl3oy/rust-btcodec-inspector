//! Inspect the system's current default audio render endpoint and decide
//! whether [`crate::watch`] is even capable of observing its codec.
//!
//! See [`observe_default_playback`] for the entry point and
//! [`CodecObservability`] for the result categories.

use windows::core::{GUID, HRESULT, PROPVARIANT};
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::Foundation::RPC_E_CHANGED_MODE;
use windows::Win32::Media::Audio::{eConsole, eRender, IMMDeviceEnumerator, MMDeviceEnumerator};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_APARTMENTTHREADED,
    STGM_READ,
};
use windows::Win32::UI::Shell::PropertiesSystem::{IPropertyStore, PROPERTYKEY};

/// `E_NOTFOUND` as returned by `IMMDeviceEnumerator::GetDefaultAudioEndpoint`
/// when there is no default endpoint of the requested kind.
///
/// This HRESULT (`0x80070490`) is not exposed as `E_NOTFOUND` from the
/// `windows` crate's `Win32::Foundation` module — it lives under namespaces
/// like `Win32::Data::HtmlHelp` and `Win32::Media::DirectShow`. Declare it
/// inline rather than enabling those features just for this constant.
const E_NOTFOUND: HRESULT = HRESULT(0x80070490_u32 as i32);

/// PnP device instance ID property key. Returns strings like
/// `USB\VID_3542&PID_3001\09B88CA9D6F088AB3C08` or
/// `BTHENUM\{...}\7&xxxxxxxx&0&BLUETOOTHDEVICE_xxxxxx`.
///
/// Defined as `DEVPKEY_Device_InstanceId` in `devpkey.h`. We declare it
/// inline because it is not re-exported by the `windows` crate's
/// `PropertiesSystem` module.
const PKEY_DEVICE_INSTANCE_ID: PROPERTYKEY = PROPERTYKEY {
    fmtid: GUID::from_u128(0x78c34fc8_104a_4aca_9ea4_524d52996e57),
    pid: 256,
};

/// Identifying information about a single audio render endpoint, gathered
/// via WASAPI + the property store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaybackDevice {
    /// Human-readable name as shown in the Windows sound mixer
    /// (e.g. `"Speakers (USB Audio Device)"`).
    pub friendly_name: String,
    /// PnP instance ID as it appears in Device Manager
    /// (e.g. `"USB\\VID_3542&PID_3001\\09B88CA9D6F088AB3C08"`).
    pub instance_id: String,
    /// USB Vendor ID parsed from `instance_id`, if it follows the
    /// `USB\VID_xxxx&PID_yyyy\...` format.
    pub vid: Option<u16>,
    /// USB Product ID parsed from `instance_id`, if it follows the
    /// `USB\VID_xxxx&PID_yyyy\...` format.
    pub pid: Option<u16>,
}

/// Whether [`crate::watch`] can produce useful results for the current
/// default playback device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodecObservability {
    /// Default output goes through Microsoft's Bluetooth stack
    /// (`BTHENUM\…` instance ID). [`crate::watch`] should report codecs.
    BluetoothMicrosoftStack(PlaybackDevice),
    /// Default output is a USB Audio Class device — almost certainly a
    /// Bluetooth transmitter that hides the radio side from Windows. The
    /// OS sees only PCM, so this tool cannot read the codec the dongle
    /// negotiated with its receiver.
    UsbAudioBypass(PlaybackDevice),
    /// Built-in speakers, HDMI audio, virtual audio cables, etc. Codec
    /// inspection is irrelevant for this kind of endpoint.
    OtherOutput(PlaybackDevice),
    /// No default playback device is configured.
    NoDevice,
}

/// Errors that can occur while inspecting the default playback device.
#[derive(Debug)]
pub enum PlaybackError {
    /// A Windows COM call failed.
    Com(windows::core::Error),
}

impl std::fmt::Display for PlaybackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Com(e) => write!(f, "COM error while inspecting default playback device: {e}"),
        }
    }
}

impl std::error::Error for PlaybackError {}

impl From<windows::core::Error> for PlaybackError {
    fn from(e: windows::core::Error) -> Self {
        Self::Com(e)
    }
}

/// Inspect the current default render endpoint and classify it.
///
/// COM is initialised in single-threaded apartment mode for the duration
/// of the call (uninitialised on the way out). If the calling thread has
/// already initialised COM in a different mode this is detected via
/// `RPC_E_CHANGED_MODE` and we proceed using the existing apartment.
pub fn observe_default_playback() -> Result<CodecObservability, PlaybackError> {
    unsafe {
        let init_hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let need_uninit = if init_hr.is_ok() {
            true
        } else if init_hr == RPC_E_CHANGED_MODE {
            false
        } else {
            return Err(PlaybackError::Com(init_hr.into()));
        };

        let result = inspect_inner();

        if need_uninit {
            CoUninitialize();
        }
        result
    }
}

unsafe fn inspect_inner() -> Result<CodecObservability, PlaybackError> {
    let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;

    let device = match enumerator.GetDefaultAudioEndpoint(eRender, eConsole) {
        Ok(d) => d,
        Err(e) if e.code() == E_NOTFOUND => return Ok(CodecObservability::NoDevice),
        Err(e) => return Err(e.into()),
    };

    let store: IPropertyStore = device.OpenPropertyStore(STGM_READ)?;
    let friendly_name = read_string_property(&store, &PKEY_Device_FriendlyName).unwrap_or_default();
    let instance_id = read_string_property(&store, &PKEY_DEVICE_INSTANCE_ID).unwrap_or_default();
    let (vid, pid) = parse_usb_vid_pid(&instance_id);

    let info = PlaybackDevice {
        friendly_name,
        instance_id: instance_id.clone(),
        vid,
        pid,
    };
    Ok(classify(&instance_id, info))
}

/// Read a string-typed property from the device's [`IPropertyStore`].
/// Returns `None` if the key is missing or the value cannot be coerced
/// to a string. `PROPVARIANT`'s `Display` impl runs `PropVariantToBSTR`
/// internally, so it handles both `VT_LPWSTR` and `VT_BSTR` payloads.
unsafe fn read_string_property(store: &IPropertyStore, key: &PROPERTYKEY) -> Option<String> {
    let value: PROPVARIANT = store.GetValue(key).ok()?;
    let s = value.to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// Parse `USB\VID_xxxx&PID_yyyy\...` into `(vid, pid)`. Returns
/// `(None, None)` for instance IDs that do not follow this pattern.
fn parse_usb_vid_pid(instance_id: &str) -> (Option<u16>, Option<u16>) {
    let upper = instance_id.to_ascii_uppercase();
    if !upper.starts_with("USB\\") {
        return (None, None);
    }
    let vid = field_after(&upper, "VID_").and_then(|s| u16::from_str_radix(s, 16).ok());
    let pid = field_after(&upper, "PID_").and_then(|s| u16::from_str_radix(s, 16).ok());
    (vid, pid)
}

/// Find a 4-character hexadecimal field that follows the given marker.
fn field_after<'a>(text: &'a str, marker: &str) -> Option<&'a str> {
    let start = text.find(marker)? + marker.len();
    let end = (start..text.len())
        .take(4)
        .take_while(|&i| text.as_bytes()[i].is_ascii_hexdigit())
        .last()?
        + 1;
    if end - start == 4 {
        Some(&text[start..end])
    } else {
        None
    }
}

fn classify(instance_id: &str, info: PlaybackDevice) -> CodecObservability {
    let upper = instance_id.to_ascii_uppercase();
    if upper.starts_with("USB\\") {
        CodecObservability::UsbAudioBypass(info)
    } else if upper.starts_with("BTHENUM\\")
        || upper.starts_with("BTHHFENUM\\")
        || upper.contains("BTHA2DP")
    {
        CodecObservability::BluetoothMicrosoftStack(info)
    } else {
        CodecObservability::OtherOutput(info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_btd700_instance_id() {
        let id = r"USB\VID_3542&PID_3001\09B88CA9D6F088AB3C08";
        assert_eq!(parse_usb_vid_pid(id), (Some(0x3542), Some(0x3001)));
    }

    #[test]
    fn parse_lowercase_instance_id() {
        let id = r"usb\vid_046d&pid_0a01\abcdef";
        assert_eq!(parse_usb_vid_pid(id), (Some(0x046D), Some(0x0A01)));
    }

    #[test]
    fn parse_non_usb_instance_id() {
        let id = r"BTHENUM\{0000110b-0000-1000-8000-00805f9b34fb}\7&abc";
        assert_eq!(parse_usb_vid_pid(id), (None, None));
    }

    #[test]
    fn classify_btd700_as_usb_bypass() {
        let info = PlaybackDevice {
            friendly_name: "USB Audio Device".into(),
            instance_id: r"USB\VID_3542&PID_3001\09B88CA9D6F088AB3C08".into(),
            vid: Some(0x3542),
            pid: Some(0x3001),
        };
        let id = info.instance_id.clone();
        match classify(&id, info) {
            CodecObservability::UsbAudioBypass(_) => {}
            other => panic!("expected UsbAudioBypass, got {other:?}"),
        }
    }

    #[test]
    fn classify_bluetooth_as_microsoft_stack() {
        let info = PlaybackDevice {
            friendly_name: "Bluetooth Headphones".into(),
            instance_id: r"BTHENUM\{0000110b-0000-1000-8000-00805f9b34fb}_LOCALMFG&0000\7&abc"
                .into(),
            vid: None,
            pid: None,
        };
        let id = info.instance_id.clone();
        match classify(&id, info) {
            CodecObservability::BluetoothMicrosoftStack(_) => {}
            other => panic!("expected BluetoothMicrosoftStack, got {other:?}"),
        }
    }

    #[test]
    fn classify_internal_as_other() {
        let info = PlaybackDevice {
            friendly_name: "Speakers".into(),
            instance_id: r"HDAUDIO\FUNC_01&VEN_10EC&DEV_0294&SUBSYS_xxxx".into(),
            vid: None,
            pid: None,
        };
        let id = info.instance_id.clone();
        match classify(&id, info) {
            CodecObservability::OtherOutput(_) => {}
            other => panic!("expected OtherOutput, got {other:?}"),
        }
    }
}
