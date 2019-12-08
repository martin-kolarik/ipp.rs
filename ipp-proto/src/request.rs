//!
//! IPP request
//!
use bytes::{BufMut, Bytes, BytesMut};
use futures::{AsyncRead, AsyncReadExt};
use log::debug;

use crate::{
    attribute::*,
    ipp::{DelimiterTag, IppVersion, Operation},
    value::*,
    IppHeader, IppPayload, StatusCode,
};

/// IPP request/response struct
pub struct IppRequestResponse {
    pub(crate) header: IppHeader,
    pub(crate) attributes: IppAttributes,
    pub(crate) payload: Option<IppPayload>,
}

impl IppRequestResponse {
    /// Create new IPP request for the operation and uri
    pub fn new(version: IppVersion, operation: Operation, uri: Option<&str>) -> IppRequestResponse {
        let hdr = IppHeader::new(version, operation as u16, 1);
        let mut retval = IppRequestResponse {
            header: hdr,
            attributes: IppAttributes::new(),
            payload: None,
        };

        retval.attributes_mut().add(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(ATTRIBUTES_CHARSET, IppValue::Charset("utf-8".to_string())),
        );
        retval.attributes_mut().add(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(ATTRIBUTES_NATURAL_LANGUAGE, IppValue::NaturalLanguage("en".to_string())),
        );

        if let Some(uri) = uri {
            retval.attributes_mut().add(
                DelimiterTag::OperationAttributes,
                IppAttribute::new(PRINTER_URI, IppValue::Uri(uri.replace("http", "ipp").to_string())),
            );
        }

        retval
    }

    /// Create response from status and id
    pub fn new_response(version: IppVersion, status: StatusCode, id: u32) -> IppRequestResponse {
        let hdr = IppHeader::new(version, status as u16, id);
        let mut retval = IppRequestResponse {
            header: hdr,
            attributes: IppAttributes::new(),
            payload: None,
        };

        retval.attributes_mut().add(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(ATTRIBUTES_CHARSET, IppValue::Charset("utf-8".to_string())),
        );
        retval.attributes_mut().add(
            DelimiterTag::OperationAttributes,
            IppAttribute::new(ATTRIBUTES_NATURAL_LANGUAGE, IppValue::NaturalLanguage("en".to_string())),
        );

        retval
    }

    /// Get IPP header
    pub fn header(&self) -> &IppHeader {
        &self.header
    }

    /// Get mutable IPP header
    pub fn header_mut(&mut self) -> &mut IppHeader {
        &mut self.header
    }

    /// Get attributes
    pub fn attributes(&self) -> &IppAttributes {
        &self.attributes
    }

    /// Get attributes
    pub fn attributes_mut(&mut self) -> &mut IppAttributes {
        &mut self.attributes
    }

    /// Get payload
    pub fn payload(&self) -> Option<&IppPayload> {
        self.payload.as_ref()
    }

    /// Get mutable payload
    pub fn payload_mut(&mut self) -> &mut Option<IppPayload> {
        &mut self.payload
    }

    /// Write request to byte array not including payload
    pub fn to_bytes(&self) -> Bytes {
        let mut buffer = BytesMut::new();
        buffer.put(self.header.to_bytes());
        buffer.put(self.attributes.to_bytes());
        buffer.freeze()
    }

    /// Convert request/response into AsyncRead including payload
    pub fn into_reader(self) -> Box<dyn AsyncRead + Send + Unpin + 'static> {
        let header = self.to_bytes();
        let cursor = futures::io::Cursor::new(header);
        debug!("IPP header size: {}", cursor.get_ref().len());

        match self.payload {
            Some(payload) => {
                debug!("Adding payload to a reader chain");
                Box::new(cursor.chain(payload.into_reader()))
            }
            _ => Box::new(cursor),
        }
    }
}
