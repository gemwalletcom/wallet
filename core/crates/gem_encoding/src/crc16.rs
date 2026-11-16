const XMODEM_POLYNOMIAL: u16 = 0x1021;

pub fn crc16_xmodem(data: &[u8]) -> u16 {
    data.iter()
        .fold(0, |crc, &byte| (0..8).fold(crc ^ (u16::from(byte) << 8), |crc, _| if crc & 0x8000 == 0 { crc << 1 } else { (crc << 1) ^ XMODEM_POLYNOMIAL }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc16_xmodem() {
        assert_eq!(crc16_xmodem(b"123456789"), 0x31c3);
        assert_eq!(crc16_xmodem(&[]), 0);
    }
}
