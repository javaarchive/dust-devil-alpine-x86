use core::fmt;

/// Pretty-prints an amount of bytes into a four-digit number followed by a unit suffix. This means
/// printed values are never more than 6 chars long.
///
/// For example, 900 will be displayed as "900B", but 1024 will be displayed as "1KB", or 1536 will
/// be displayed as "1.5KB". The suffixes used can be K, M, G, T, P, and E.
pub struct PrettyByteDisplayer(pub usize);

const BYTE_COUNT_SUFFIXES: &[char] = &['K', 'M', 'G', 'T', 'P', 'E'];

impl fmt::Display for PrettyByteDisplayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value_b = self.0;

        if value_b < 1000 {
            return write!(f, "{value_b}B");
        }

        let mut suffix_index = 0;
        let mut value_prev = value_b;
        let mut value = value_prev / 1024;
        let mut multiplier = 1024;

        while suffix_index < BYTE_COUNT_SUFFIXES.len() - 1 && value > 1000 {
            suffix_index += 1;
            value_prev = value;
            value /= 1024;
            multiplier *= 1024;
        }

        let suffix_char = BYTE_COUNT_SUFFIXES[suffix_index];

        write!(f, "{value}")?;

        if value * multiplier != value_b {
            if value < 10 {
                let decimal_digits = (((value_prev as f32 / 1024.0).fract() * 100.0) as i8).clamp(0, 99) as u8;
                write!(f, ".{decimal_digits:0>2}")?;
            } else if value < 100 {
                let decimal_digits = (((value_prev as f32 / 1024.0).fract() * 10.0) as i8).clamp(0, 9) as u8;
                write!(f, ".{decimal_digits:0>1}")?;
            }
        }

        write!(f, "{suffix_char}B")
    }
}
