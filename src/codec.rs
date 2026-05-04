use std::fmt;

/// Identifies a Bluetooth A2DP audio codec by its standard codec ID and,
/// for vendor-specific codecs, the vendor ID and vendor codec ID.
///
/// Sources for the codec ID -> name mappings:
/// - bluez-alsa: <https://github.com/arkq/bluez-alsa/blob/master/src/shared/bluetooth-a2dp.h>
/// - BlueZ: <https://github.com/bluez/bluez/blob/master/profiles/audio/a2dp-codecs.h>
/// - Bluetooth SIG company identifiers: <https://www.bluetooth.com/specifications/assigned-numbers/>
/// - Helge Klein, "How to Check Which Bluetooth A2DP Audio Codec Is Used on Windows"
/// - imbushuo/BluetoothAudioCodecInspector (the original C# tool)
///
/// # Example
/// ```
/// use btcodec_inspector::A2dpCodec;
/// assert_eq!(A2dpCodec::new(0x02, 0, 0).to_string(), "AAC");
/// assert_eq!(A2dpCodec::new(0xFF, 0x012D, 0x00AA).to_string(), "Sony LDAC");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct A2dpCodec {
    /// A2DP standard codec ID (e.g. `0x00` = SBC, `0x02` = AAC, `0xFF` = vendor-specific).
    pub standard_codec_id: u8,
    /// Bluetooth SIG company identifier. Only meaningful when `standard_codec_id == 0xFF`.
    pub vendor_id: i32,
    /// Vendor-defined codec ID. Only meaningful when `standard_codec_id == 0xFF`.
    pub vendor_codec_id: i32,
}

impl A2dpCodec {
    pub fn new(standard_codec_id: u8, vendor_id: i32, vendor_codec_id: i32) -> Self {
        Self {
            standard_codec_id,
            vendor_id,
            vendor_codec_id,
        }
    }
}

/// A2DP standard codec IDs (low byte of the codec descriptor).
///
/// `0x05`–`0xFE` are reserved by the A2DP spec; `0xFF` selects a
/// vendor-specific codec described by `vendor_id` + `vendor_codec_id`.
const STANDARD_CODECS: &[(u8, &str)] = &[
    (0x00, "SBC"),
    (0x01, "MP3"),
    (0x02, "AAC"),
    (0x03, "MPEG-D USAC"),
    (0x04, "ATRAC"),
];

/// Vendor-specific A2DP codecs, keyed by `(vendor_id, vendor_codec_id)`.
///
/// Where multiple sources disagree, bluez-alsa is treated as canonical.
const VENDOR_CODECS: &[(i32, i32, &str)] = &[
    // --- APT Ltd. (vendor 0x004F) ---
    (0x004F, 0x0001, "aptX"),
    // --- Qualcomm Technologies International / CSR (vendor 0x000A) ---
    (0x000A, 0x0001, "Qualcomm/CSR FastStream"),
    (0x000A, 0x0002, "Qualcomm/CSR aptX Low Latency"),
    (0x000A, 0x0104, "Qualcomm/CSR True Wireless Stereo v3 (AAC)"),
    (0x000A, 0x0105, "Qualcomm/CSR True Wireless Stereo v3 (MP3)"),
    (
        0x000A,
        0x0106,
        "Qualcomm/CSR True Wireless Stereo v3 (aptX)",
    ),
    // --- Qualcomm Technologies, Inc. (vendor 0x00D7) ---
    (0x00D7, 0x0002, "Qualcomm aptX Low Latency"),
    (0x00D7, 0x0024, "Qualcomm aptX HD"),
    (0x00D7, 0x0025, "Qualcomm aptX TWS+"),
    // aptX Adaptive carries aptX Lossless mode internally — there is no
    // separate codec ID for Lossless; it negotiates over Adaptive.
    (
        0x00D7,
        0x00AD,
        "Qualcomm aptX Adaptive (incl. Lossless mode)",
    ),
    // --- Samsung Electronics (vendor 0x0075) ---
    (0x0075, 0x0102, "Samsung HD"),
    (0x0075, 0x0103, "Samsung Scalable Codec"),
    // --- Sony (vendor 0x012D) ---
    (0x012D, 0x00AA, "Sony LDAC"),
    // --- Savitech / LHDC family (vendor 0x053A) ---
    (0x053A, 0x484C, "Savitech LHDC v1"),
    (0x053A, 0x4C32, "Savitech LHDC v2"),
    // bluez-alsa lists "LHDC-v3", "LHDC-v4" and "LLAC" as aliases for the
    // same codec ID.
    (0x053A, 0x4C33, "Savitech LHDC v3 (a.k.a. v4 / LLAC)"),
    (0x053A, 0x4C35, "Savitech LHDC v5"),
    (0x053A, 0x4C4C, "Savitech LHDC LL"),
    // --- Google (vendor 0x00E0) ---
    (0x00E0, 0x0001, "Google Opus"),
    // --- The Linux Foundation (vendor 0x05F1) ---
    (0x05F1, 0x1005, "PipeWire Opus"),
    // --- Fraunhofer IIS (vendor 0x08A9) ---
    (0x08A9, 0x0001, "Fraunhofer LC3plus"),
    // --- Shenzhen CESI (vendor 0x0CCF) — used by BES SDK for Huawei L2HC ---
    (0x0CCF, 0xCA01, "Huawei L2HC"),
];

/// Bluetooth SIG company identifiers we recognise. Used to give a friendlier
/// "Unknown Codec from <vendor>" message when only the vendor is known.
///
/// Source: <https://www.bluetooth.com/specifications/assigned-numbers/>
const VENDOR_NAMES: &[(i32, &str)] = &[
    (0x0002, "Intel Corp."),
    (0x000A, "Qualcomm Technologies International (CSR)"),
    (0x000F, "Broadcom Corp."),
    (0x004C, "Apple, Inc."),
    (0x004F, "APT Ltd."),
    (0x005D, "Realtek Semiconductor Corp."),
    (0x0075, "Samsung Electronics Co. Ltd."),
    (0x00D7, "Qualcomm Technologies, Inc."),
    (0x00E0, "Google"),
    (0x012D, "Sony Corporation"),
    (0x013A, "Tencent Holdings Ltd."),
    (0x027D, "HUAWEI Technologies Co., Ltd."),
    (0x053A, "Savitech Corp."),
    (0x05F1, "The Linux Foundation"),
    (0x08A9, "Fraunhofer IIS"),
    (0x0CCF, "Shenzhen CESI Information Technology Co., Ltd."),
];

fn standard_codec_name(id: u8) -> Option<&'static str> {
    STANDARD_CODECS
        .iter()
        .find(|(k, _)| *k == id)
        .map(|(_, name)| *name)
}

fn vendor_codec_name(vendor_id: i32, vendor_codec_id: i32) -> Option<&'static str> {
    VENDOR_CODECS
        .iter()
        .find(|(v, c, _)| *v == vendor_id && *c == vendor_codec_id)
        .map(|(_, _, name)| *name)
}

fn vendor_name(vendor_id: i32) -> Option<&'static str> {
    VENDOR_NAMES
        .iter()
        .find(|(k, _)| *k == vendor_id)
        .map(|(_, name)| *name)
}

impl fmt::Display for A2dpCodec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 1. Known standard codec.
        if let Some(name) = standard_codec_name(self.standard_codec_id) {
            return f.write_str(name);
        }

        // 2. Reserved standard codec ID (0x05–0xFE).
        if self.standard_codec_id != 0xFF {
            return write!(
                f,
                "Reserved standard codec ID 0x{:02X} (vendor 0x{:04X}, codec 0x{:04X})",
                self.standard_codec_id, self.vendor_id, self.vendor_codec_id,
            );
        }

        // 3. Known vendor-specific codec.
        if let Some(name) = vendor_codec_name(self.vendor_id, self.vendor_codec_id) {
            return f.write_str(name);
        }

        // 4. Unknown vendor-specific codec; include vendor name if known.
        match vendor_name(self.vendor_id) {
            Some(name) => write!(
                f,
                "Unknown codec from {} (vendor 0x{:04X}, codec 0x{:04X})",
                name, self.vendor_id, self.vendor_codec_id,
            ),
            None => write!(
                f,
                "Unknown codec (vendor 0x{:04X}, codec 0x{:04X})",
                self.vendor_id, self.vendor_codec_id,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Standard codecs ---

    #[test]
    fn standard_sbc() {
        assert_eq!(A2dpCodec::new(0x00, 0, 0).to_string(), "SBC");
    }

    #[test]
    fn standard_mp3() {
        assert_eq!(A2dpCodec::new(0x01, 0, 0).to_string(), "MP3");
    }

    #[test]
    fn standard_aac() {
        assert_eq!(A2dpCodec::new(0x02, 0, 0).to_string(), "AAC");
    }

    #[test]
    fn standard_usac() {
        assert_eq!(A2dpCodec::new(0x03, 0, 0).to_string(), "MPEG-D USAC");
    }

    #[test]
    fn standard_atrac() {
        assert_eq!(A2dpCodec::new(0x04, 0, 0).to_string(), "ATRAC");
    }

    // --- aptX family ---

    #[test]
    fn aptx() {
        assert_eq!(A2dpCodec::new(0xFF, 0x004F, 0x0001).to_string(), "aptX");
    }

    #[test]
    fn aptx_hd() {
        assert_eq!(
            A2dpCodec::new(0xFF, 0x00D7, 0x0024).to_string(),
            "Qualcomm aptX HD"
        );
    }

    #[test]
    fn aptx_ll_csr() {
        assert_eq!(
            A2dpCodec::new(0xFF, 0x000A, 0x0002).to_string(),
            "Qualcomm/CSR aptX Low Latency"
        );
    }

    #[test]
    fn aptx_ll_qualcomm() {
        assert_eq!(
            A2dpCodec::new(0xFF, 0x00D7, 0x0002).to_string(),
            "Qualcomm aptX Low Latency"
        );
    }

    #[test]
    fn aptx_tws_plus() {
        assert_eq!(
            A2dpCodec::new(0xFF, 0x00D7, 0x0025).to_string(),
            "Qualcomm aptX TWS+"
        );
    }

    #[test]
    fn aptx_adaptive() {
        assert_eq!(
            A2dpCodec::new(0xFF, 0x00D7, 0x00AD).to_string(),
            "Qualcomm aptX Adaptive (incl. Lossless mode)"
        );
    }

    #[test]
    fn faststream() {
        assert_eq!(
            A2dpCodec::new(0xFF, 0x000A, 0x0001).to_string(),
            "Qualcomm/CSR FastStream"
        );
    }

    #[test]
    fn csr_tws_v3_aac() {
        assert_eq!(
            A2dpCodec::new(0xFF, 0x000A, 0x0104).to_string(),
            "Qualcomm/CSR True Wireless Stereo v3 (AAC)"
        );
    }

    #[test]
    fn csr_tws_v3_mp3() {
        assert_eq!(
            A2dpCodec::new(0xFF, 0x000A, 0x0105).to_string(),
            "Qualcomm/CSR True Wireless Stereo v3 (MP3)"
        );
    }

    #[test]
    fn csr_tws_v3_aptx() {
        assert_eq!(
            A2dpCodec::new(0xFF, 0x000A, 0x0106).to_string(),
            "Qualcomm/CSR True Wireless Stereo v3 (aptX)"
        );
    }

    // --- LDAC ---

    #[test]
    fn ldac() {
        assert_eq!(
            A2dpCodec::new(0xFF, 0x012D, 0x00AA).to_string(),
            "Sony LDAC"
        );
    }

    // --- Samsung ---

    #[test]
    fn samsung_hd() {
        assert_eq!(
            A2dpCodec::new(0xFF, 0x0075, 0x0102).to_string(),
            "Samsung HD"
        );
    }

    #[test]
    fn samsung_scalable() {
        assert_eq!(
            A2dpCodec::new(0xFF, 0x0075, 0x0103).to_string(),
            "Samsung Scalable Codec"
        );
    }

    // --- LHDC family ---

    #[test]
    fn lhdc_v1() {
        assert_eq!(
            A2dpCodec::new(0xFF, 0x053A, 0x484C).to_string(),
            "Savitech LHDC v1"
        );
    }

    #[test]
    fn lhdc_v2() {
        assert_eq!(
            A2dpCodec::new(0xFF, 0x053A, 0x4C32).to_string(),
            "Savitech LHDC v2"
        );
    }

    #[test]
    fn lhdc_v3() {
        assert_eq!(
            A2dpCodec::new(0xFF, 0x053A, 0x4C33).to_string(),
            "Savitech LHDC v3 (a.k.a. v4 / LLAC)"
        );
    }

    #[test]
    fn lhdc_v5() {
        assert_eq!(
            A2dpCodec::new(0xFF, 0x053A, 0x4C35).to_string(),
            "Savitech LHDC v5"
        );
    }

    #[test]
    fn lhdc_ll() {
        assert_eq!(
            A2dpCodec::new(0xFF, 0x053A, 0x4C4C).to_string(),
            "Savitech LHDC LL"
        );
    }

    // --- Opus / LC3plus / L2HC ---

    #[test]
    fn google_opus() {
        assert_eq!(
            A2dpCodec::new(0xFF, 0x00E0, 0x0001).to_string(),
            "Google Opus"
        );
    }

    #[test]
    fn pipewire_opus() {
        assert_eq!(
            A2dpCodec::new(0xFF, 0x05F1, 0x1005).to_string(),
            "PipeWire Opus"
        );
    }

    #[test]
    fn lc3plus() {
        assert_eq!(
            A2dpCodec::new(0xFF, 0x08A9, 0x0001).to_string(),
            "Fraunhofer LC3plus"
        );
    }

    #[test]
    fn huawei_l2hc() {
        assert_eq!(
            A2dpCodec::new(0xFF, 0x0CCF, 0xCA01).to_string(),
            "Huawei L2HC"
        );
    }

    // --- Unknown / reserved ---

    #[test]
    fn reserved_standard_id() {
        assert_eq!(
            A2dpCodec::new(0x10, 0, 0).to_string(),
            "Reserved standard codec ID 0x10 (vendor 0x0000, codec 0x0000)"
        );
    }

    #[test]
    fn unknown_vendor_codec_known_vendor() {
        // Apple Inc. (vendor 0x004C) with an unmapped codec ID.
        assert_eq!(
            A2dpCodec::new(0xFF, 0x004C, 0x8001).to_string(),
            "Unknown codec from Apple, Inc. (vendor 0x004C, codec 0x8001)"
        );
    }

    #[test]
    fn unknown_vendor_codec_unknown_vendor() {
        assert_eq!(
            A2dpCodec::new(0xFF, 0xDEAD, 0xBEEF).to_string(),
            "Unknown codec (vendor 0xDEAD, codec 0xBEEF)"
        );
    }

    // --- Sanity: every entry in VENDOR_CODECS resolves via vendor_codec_name ---

    #[test]
    fn vendor_table_self_consistent() {
        for (vid, cid, name) in VENDOR_CODECS {
            let resolved = vendor_codec_name(*vid, *cid).unwrap_or("");
            assert_eq!(
                resolved, *name,
                "table entry mismatch: {vid:#06x}:{cid:#06x}"
            );
        }
    }
}
