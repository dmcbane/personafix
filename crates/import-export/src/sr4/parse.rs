/// SR4 character file (.chum) parser.
///
/// SR4 files are UTF-16 LE encoded with BOM. Decode to UTF-8 first,
/// then parse XML. Attribute schema uses `<value>` instead of `<base>+<karma>`.

pub fn decode_utf16(bytes: &[u8]) -> String {
    let (encoding, bom_len) = encoding_rs::Encoding::for_bom(bytes)
        .unwrap_or((encoding_rs::UTF_8, 0));
    let (cow, _, _) = encoding.decode(&bytes[bom_len..]);
    cow.into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_utf8_passthrough() {
        let xml = b"<?xml version=\"1.0\"?><character></character>";
        let decoded = decode_utf16(xml);
        assert!(decoded.contains("<character>"));
    }
}
