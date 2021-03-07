use std::convert::TryFrom;

use crate::{checksum, Error};

const RESPONSE_SIZE_NONE: usize = 0;
const RESPONSE_SIZE_GENERAL: usize = 4;
const RESPONSE_SIZE_EXTENDED: usize = 7;
const RESPONSE_SIZE_QUERY: usize = 18;
const RESPONSE_SIZE_MAX: usize = RESPONSE_SIZE_QUERY;

/// The types of response returned by the device.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResponseKind {
    /// 0 bytes, no response
    None,
    /// 4 bytes, common response
    General,
    /// 7 bytes, data response
    Extended,
    /// 18 bytes, response of "query" command
    Query,
}

/// Response message.
#[derive(Debug, Clone, PartialEq)]
pub struct Response {
    kind: ResponseKind,
    data: [u8; RESPONSE_SIZE_MAX],
}

impl Response {
    /// The kind of the response
    pub fn kind(&self) -> ResponseKind {
        self.kind
    }

    /// The content of the response,
    pub fn bytes(&self) -> &[u8] {
        match self.kind {
            ResponseKind::None => &self.data[..RESPONSE_SIZE_NONE],
            ResponseKind::General => &self.data[..RESPONSE_SIZE_GENERAL],
            ResponseKind::Extended => &self.data[..RESPONSE_SIZE_EXTENDED],
            ResponseKind::Query => &self.data[..RESPONSE_SIZE_QUERY],
        }
    }

    /// Verify the checksum of the response. The argument is the checksum value
    /// of the command that caused this reply. It is needed for General and
    /// Query types, for other types this value is ignored.
    pub fn checksum_is_valid(&self, cmd_cksm: u8) -> bool {
        match self.kind {
            ResponseKind::None => true,
            ResponseKind::General => checksum(&[cmd_cksm, self.data[2]]) == self.data[3],
            ResponseKind::Extended => checksum(&self.data[1..6]) == self.data[6],
            ResponseKind::Query => {
                ((checksum(&self.data[1..17]) as u32 + cmd_cksm as u32) & 0xFF) as u8
                    == self.data[17]
            }
        }
    }
}

impl TryFrom<&[u8]> for Response {
    type Error = Error;

    /// The conversion will be successful only if the size of the slice correponds
    /// to one of the response types: 0, 4, 7 or 18 bytes.
    /// Note: the checksum of the Response is not validated.
    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        let mut response = Response {
            kind: ResponseKind::None,
            data: [0; RESPONSE_SIZE_MAX],
        };
        match value.len() {
            RESPONSE_SIZE_NONE => Ok(response),

            RESPONSE_SIZE_GENERAL => {
                response.kind = ResponseKind::General;
                response.data[..RESPONSE_SIZE_GENERAL].copy_from_slice(value);
                Ok(response)
            }

            RESPONSE_SIZE_EXTENDED => {
                response.kind = ResponseKind::Extended;
                response.data[..RESPONSE_SIZE_EXTENDED].copy_from_slice(value);
                Ok(response)
            }

            RESPONSE_SIZE_QUERY => {
                response.kind = ResponseKind::Query;
                response.data[..RESPONSE_SIZE_QUERY].copy_from_slice(value);
                Ok(response)
            }

            l => {
                let msg = format!("Invalid response length {}", l);
                Err(Error::InvalidValue(msg))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_response_bytes() {
        let mut response = Response {
            kind: ResponseKind::None,
            data: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17],
        };
        assert_eq!(response.bytes(), &[]);

        response.kind = ResponseKind::General;
        assert_eq!(response.bytes(), &[0, 1, 2, 3]);

        response.kind = ResponseKind::Extended;
        assert_eq!(response.bytes(), &[0, 1, 2, 3, 4, 5, 6]);

        response.kind = ResponseKind::Query;
        assert_eq!(
            response.bytes(),
            &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17]
        );
    }

    #[allow(unused_variables)]
    #[test]
    fn test_verify_checksum() {
        // From examples in Peldo D spec

        // General, p.20
        let dce = [0xffu8, 0x01, 0x00, 0x07, 0x00, 0x22, 0x2a];
        let dte = [0xffu8, 0x01, 0x00, 0x2a];
        let response = Response::try_from(&dte[..]).unwrap();
        assert_eq!(response.kind(), ResponseKind::General);
        assert!(response.checksum_is_valid(0x2a));

        // Extended, p.20
        let dce = [0xffu8, 0x01, 0x00, 0x51, 0x00, 0x00, 0x52];
        let dte = [0xffu8, 0x01, 0x00, 0x59, 0x00, 0x00, 0x5a];
        let response = Response::try_from(&dte[..]).unwrap();
        assert_eq!(response.kind(), ResponseKind::Extended);
        assert!(response.checksum_is_valid(0));

        // Query, p.21-22
        let dte = [0xffu8, 0x01, 0x00, 0x45, 0x00, 0x00, 0x46];
        let dce = [
            0xffu8, 0x01, 0x44, 0x44, 0x35, 0x33, 0x43, 0x42, 0x57, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x13,
        ];
        let response = Response::try_from(&dce[..]).unwrap();
        assert_eq!(response.kind(), ResponseKind::Query);
        assert!(response.checksum_is_valid(0x46));
    }
}
