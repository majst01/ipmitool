//! IPMI checksum helper.

/// Compute the IPMI 2's-complement checksum over `data`.
pub fn ipmi_csum(data: &[u8]) -> u8 {
    let mut c: u8 = 0;
    for &b in data {
        c = c.wrapping_add(b);
    }
    c.wrapping_neg()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_makes_sum_zero() {
        let data = [0x20u8, 0x18, 0x04, 0x81];
        let c = ipmi_csum(&data);
        let sum = data
            .iter()
            .fold(0u8, |a, &b| a.wrapping_add(b))
            .wrapping_add(c);
        assert_eq!(sum, 0);
    }

    #[test]
    fn empty_checksum_is_zero() {
        assert_eq!(ipmi_csum(&[]), 0);
    }
}
