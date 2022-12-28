const BASE32: [&str; 32] = [
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e", "f", "g", "h", "j",
    "k", "m", "n", "p", "q", "r", "s", "t", "v", "w", "x", "y", "z",
];
const MAX_ROW: usize = 32 * 32 * 32;

pub fn to_base32(i: usize) -> String {
    let mut i = i;
    if i > MAX_ROW {
        "".to_owned()
    } else {
        let x = i / (32 * 32);
        i %= 32 * 32;
        let y = i / 32;
        let z = i % 32;

        if x == 0 && y == 0 {
            BASE32[z].to_owned()
        } else if x == 0 {
            format!("{}{}", BASE32[y], BASE32[z])
        } else {
            format!("{}{}{}", BASE32[x], BASE32[y], BASE32[z])
        }
    }
}

pub fn from_base32(s: &str) -> Option<usize> {
    if s.len() > 3 || s.is_empty() {
        None
    } else {
        let mut result = vec![];
        for c in s.chars() {
            let index = BASE32.iter().position(|&x| x == c.to_string());
            if let Some(index) = index {
                result.push(index);
            } else {
                return None;
            }
        }
        match result.len() {
            3 => Some((result[0] * 32 * 32) + (result[1] * 32) + (result[2])),
            2 => Some((result[0] * 32) + (result[1])),
            1 => Some(result[0]),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_base32() {
        assert_eq!(to_base32(3), "3".to_string());
        assert_eq!(to_base32(35), "13".to_string());
        assert_eq!(to_base32(1024), "100".to_string());
        assert_eq!(to_base32(5032), "4x8".to_string());
        assert_eq!(to_base32(30000), "x9g".to_string());
    }

    #[test]
    fn test_from_base32() {
        assert_eq!(from_base32("3"), Some(3));
        assert_eq!(from_base32("13"), Some(35));
        assert_eq!(from_base32("100"), Some(1024));
        assert_eq!(from_base32("4x8"), Some(5032));
        assert_eq!(from_base32("x9g"), Some(30000));
    }
}
