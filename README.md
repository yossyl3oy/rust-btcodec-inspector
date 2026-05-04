# btcodec-inspector

Rust port of [imbushuo/BluetoothAudioCodecInspector](https://github.com/imbushuo/BluetoothAudioCodecInspector). Inspects which Bluetooth A2DP audio codec is currently in use on Windows by subscribing to the `Microsoft.Windows.Bluetooth.BthA2dp` ETW provider — a workaround because Windows does not expose the active codec through any public API.

Distributed as both a CLI binary and a library crate.

## Requirements

- Windows 10 / 11 (this crate is Windows-only; non-Windows builds fail with `compile_error!`)
- Administrator privilege (ETW realtime sessions require it)

## CLI

### Download

Pre-built `.exe` for `x86_64-pc-windows-msvc` is attached to each [GitHub Release](../../releases). Grab the latest one and run from an elevated prompt.

### Usage

```cmd
btcodec-inspector.exe
```

Start audio playback over Bluetooth. The codec for each `A2dpStreaming` event will be printed:

```
A2DP Streaming event. Codec: AAC
```

Press `Ctrl+C` to stop.

### Debug mode

If nothing appears after the initial banner (i.e. no `A2DP Streaming event.` lines while audio is playing), set `BTCODEC_DEBUG=1` to have every observed ETW event dumped to stderr along with the parse result for the three codec fields. This is the quickest way to tell whether events are arriving at all and whether the field names still match what your Windows version emits.

```cmd
set BTCODEC_DEBUG=1
btcodec-inspector.exe
```

## Library

```toml
[dependencies]
btcodec-inspector = { version = "0.2", default-features = false }
```

`default-features = false` opts out of the `cli` feature, which is only needed by the CLI binary (it pulls in `ctrlc`). The library itself only depends on `ferrisetw` and `is_elevated` on Windows.

```rust
use btcodec_inspector::watch;

let _watcher = watch(|codec| {
    println!("Codec: {}", codec);
}).expect("failed to start ETW session");

// Keep `_watcher` alive while you want events. Drop to stop.
std::thread::sleep(std::time::Duration::from_secs(60));
```

The callback is `FnMut`, so captured state can be mutated directly without `Arc<Mutex<...>>`:

```rust
use btcodec_inspector::{watch, A2dpCodec};

let mut history: Vec<A2dpCodec> = Vec::new();
let _watcher = watch(move |codec| {
    history.push(codec);
}).unwrap();
```

The `A2dpCodec` type itself is cross-platform, so the codec ID -> name mapping can be used on macOS / Linux too if you have raw IDs from another source.

## Supported codecs

**Standard** (A2DP spec, low-byte codec ID): SBC, MP3, AAC, MPEG-D USAC, ATRAC.

**Vendor-specific**:

| Family | Codecs |
|---|---|
| Qualcomm / CSR | aptX, aptX HD, aptX LL (CSR + Qualcomm variants), aptX TWS+, aptX Adaptive (incl. Lossless mode), FastStream, True Wireless Stereo v3 (AAC / MP3 / aptX) |
| Sony | LDAC |
| Samsung | Samsung HD, Samsung Scalable Codec |
| Savitech (LHDC) | LHDC v1, v2, v3 (= v4 / LLAC), v5, LL |
| Other | Google Opus, PipeWire Opus, Fraunhofer LC3plus, Huawei L2HC |

For unrecognised vendor codecs, the company name is included when the Bluetooth SIG company identifier is known (e.g. `Unknown codec from Apple, Inc. (vendor 0x004C, codec 0x8001)`).

Mappings are sourced from [bluez-alsa](https://github.com/arkq/bluez-alsa), [BlueZ](https://github.com/bluez/bluez), the [Bluetooth SIG company identifiers](https://www.bluetooth.com/specifications/assigned-numbers/), [Helge Klein's writeup](https://helgeklein.com/blog/how-to-check-which-bluetooth-a2dp-audio-codec-is-used-on-windows/), and the original [imbushuo/BluetoothAudioCodecInspector](https://github.com/imbushuo/BluetoothAudioCodecInspector).

## Build

On Windows:

```cmd
cargo build --release
```

Cross-compile from macOS / Linux:

```sh
rustup target add x86_64-pc-windows-msvc
cargo check --target x86_64-pc-windows-msvc
# To produce a .exe, install cargo-xwin and run:
#   cargo install cargo-xwin
#   cargo xwin build --release --target x86_64-pc-windows-msvc
```

## Notes

- A2DP streaming events are not always emitted in real time. Format information may appear during the playback session or only at the end. This is a Windows Bluetooth Audio Stack limitation.
- Only works when the Bluetooth chipset is using Microsoft's Bluetooth stack (the case for ~all modern Windows PCs). 3rd-party stacks (legacy WIDCOMM / BTW, old Toshiba stack, etc.) bypass this provider and emit no events.
- LE Audio (LC3 / LC3plus over LE Audio) uses a different ETW provider and is not covered.

## License

MIT — see [LICENSE](LICENSE).
