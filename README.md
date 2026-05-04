# btcodec-inspector

Rust port of [imbushuo/BluetoothAudioCodecInspector](https://github.com/imbushuo/BluetoothAudioCodecInspector). A Windows-only CLI utility that inspects which Bluetooth A2DP audio codec is currently in use, by subscribing to the `Microsoft.Windows.Bluetooth.BthA2dp` ETW provider.

Useful because Windows does not expose the active codec through any public API.

## Requirements

- Windows 10 / 11
- Administrator privilege (ETW realtime sessions require it)

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

## Download

Pre-built `.exe` for `x86_64-pc-windows-msvc` is attached to each [GitHub Release](../../releases). Grab the latest one and run from an elevated prompt.

## Usage

Open an elevated Command Prompt and run:

```cmd
btcodec-inspector.exe
```

Start audio playback over Bluetooth. The codec for each `A2dpStreaming` event will be printed:

```
A2DP Streaming event. Codec: AAC
```

Press `Ctrl+C` to stop.

## Notes

- A2DP streaming events are not always emitted in real time. Format information may appear during the playback session or only at the end. This is a Windows Bluetooth Audio Stack limitation.
- Codec ID mapping is taken from the [original project](https://github.com/imbushuo/BluetoothAudioCodecInspector) and [Helge Klein's writeup](https://helgeklein.com/blog/how-to-check-which-bluetooth-a2dp-audio-codec-is-used-on-windows/).

## License

MIT — see [LICENSE](LICENSE).
