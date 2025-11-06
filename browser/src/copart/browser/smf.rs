use common::io::error::GeneralError;
use std::array::TryFromSliceError;

pub struct SmfSizesDecoder;

#[derive(Debug)]
pub struct SmfSizes {
    pub header_bytes: usize,
    pub msg_bytes: usize,
}

impl SmfSizesDecoder {
    const KNOWN_RECEIVABLE_PROTOCOLS: [u32; 10] = [3, 9, 10, 11, 12, 13, 14, 15, 19, 20];

    pub fn decode<'a>(data: impl Into<&'a [u8]>) -> Result<SmfSizes, GeneralError> {
        let data = data.into();
        let mut off = 0;

        let word_long1 = Self::four_byte_to_uint(data, off)?;
        off += 4;
        let word_long2 = Self::four_byte_to_uint(data, off)?;
        off += 4;

        let smf_version = Self::extract_bits(word_long1, 3, 24);
        let protocol = Self::extract_bits(word_long1, 6, 16);

        let (hdr_len_bytes, bytes_read, reported_msg_len) = match smf_version {
            2 => {
                let hdr_len_words = Self::extract_bits(word_long1, 12, 0);
                let hdr_len_bytes = hdr_len_words * 4;
                let reported_msg_len = Self::extract_bits(word_long2, 24, 0);
                (hdr_len_bytes, 8, reported_msg_len)
            }
            3 => {
                let hdr_len_bytes = word_long2;
                let word_long3 = Self::four_byte_to_uint(data, off)?;
                let reported_msg_len = word_long3;
                (hdr_len_bytes, 12, reported_msg_len)
            }
            v => {
                return Err(GeneralError::Smf(format!(
                    "unsupported SMF version: version = `{v}`"
                )));
            }
        };

        let msg_len = if Self::KNOWN_RECEIVABLE_PROTOCOLS.contains(&protocol) {
            reported_msg_len - hdr_len_bytes
        } else {
            reported_msg_len - bytes_read
        };

        Ok(SmfSizes {
            header_bytes: hdr_len_bytes as usize,
            msg_bytes: msg_len as usize,
        })
    }

    fn four_byte_to_uint(data: &[u8], offset: usize) -> Result<u32, GeneralError> {
        if offset + 4 > data.len() {
            return Err(GeneralError::Smf(format!(
                "slice overflow error: data_len = `{}`, offset = `{offset}`",
                data.len()
            )));
        }
        let slice = &data[offset..offset + 4];
        Ok(u32::from_be_bytes(slice.try_into().map_err(
            |e: TryFromSliceError| {
                GeneralError::Smf(format!("slice conversion error: `{}`", e.to_string()))
            },
        )?))
    }

    fn extract_bits(x: u32, num_bits: u32, shift_right: u32) -> u32 {
        (x >> shift_right) & ((1 << num_bits) - 1)
    }
}
