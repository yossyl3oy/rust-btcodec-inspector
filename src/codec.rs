use std::fmt;

pub struct A2dpCodec {
    pub standard_codec_id: u8,
    pub vendor_id: i32,
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

impl fmt::Display for A2dpCodec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.standard_codec_id {
            0x00 => return f.write_str("SBC"),
            0x01 => return f.write_str("MP3"),
            0x02 => return f.write_str("AAC"),
            0x04 => return f.write_str("ATRAC"),
            _ => {}
        }

        if self.standard_codec_id != 0xFF {
            return write!(
                f,
                "Unknown Codec (Invalid Vendor): {} {}:{}",
                self.standard_codec_id, self.vendor_id, self.vendor_codec_id
            );
        }

        // Credits: https://helgeklein.com/blog/how-to-check-which-bluetooth-a2dp-audio-codec-is-used-on-windows/
        let name: Option<&str> = match (self.vendor_id, self.vendor_codec_id) {
            (0x004F, 0x0001) => Some("Qualcomm/CSR aptX"),
            (0x00D7, 0x0024) => Some("Qualcomm/CSR aptX HD"),
            (0x00D7, 0x0002) | (0x000A, 0x0002) => Some("Qualcomm/CSR aptX LL"),
            (0x000A, 0x0001) => Some("Qualcomm/CSR FastStream"),
            (0x000A, 0x0104) => Some("Qualcomm/CSR True Wireless Stereo v3, AAC"),
            (0x000A, 0x0105) => Some("Qualcomm/CSR True Wireless Stereo v3, MP3"),
            (0x000A, 0x0106) => Some("Qualcomm/CSR True Wireless Stereo v3, aptX"),
            (0x012D, 0x00AA) => Some("Sony LDAC"),
            (0x0075, 0x0102) => Some("Samsung HD"),
            (0x0075, 0x0103) => Some("Samsung Scalable Codec"),
            (0x053A, 0x484C) => Some("Savitech LHDC"),
            _ => None,
        };

        match name {
            Some(n) => f.write_str(n),
            None => write!(
                f,
                "Unknown Codec: {} {}:{}",
                self.standard_codec_id, self.vendor_id, self.vendor_codec_id
            ),
        }
    }
}
