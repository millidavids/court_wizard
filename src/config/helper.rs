/// Helper function to calculate aspect ratio string from width and height
pub(super) fn calculate_aspect_ratio(width: u32, height: u32) -> String {
    let ratio = width as f32 / height as f32;
    match (ratio * 100.0).round() as u32 {
        177 => "16:9".to_string(),
        160 => "16:10".to_string(),
        133 => "4:3".to_string(),
        233 => "21:9".to_string(),
        _ => format!(
            "{}:{}",
            width / gcd(width, height),
            height / gcd(width, height)
        ),
    }
}

/// Helper function to calculate greatest common divisor
fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_aspect_ratio_common_ratios() {
        // Test common aspect ratios are recognized
        assert_eq!(calculate_aspect_ratio(1920, 1080), "16:9");
        assert_eq!(calculate_aspect_ratio(1280, 720), "16:9");
        assert_eq!(calculate_aspect_ratio(3840, 2160), "16:9");

        assert_eq!(calculate_aspect_ratio(1920, 1200), "16:10");
        assert_eq!(calculate_aspect_ratio(1280, 800), "16:10");

        assert_eq!(calculate_aspect_ratio(1024, 768), "4:3");
        assert_eq!(calculate_aspect_ratio(1600, 1200), "4:3");

        // 21:9 aspect ratio - test resolutions that actually produce ratio ~2.33
        assert_eq!(calculate_aspect_ratio(2100, 900), "21:9");
        assert_eq!(calculate_aspect_ratio(4200, 1800), "21:9");
    }

    #[test]
    fn test_calculate_aspect_ratio_custom_ratios() {
        // Test uncommon ratios get simplified
        assert_eq!(calculate_aspect_ratio(800, 600), "4:3"); // Should match common ratio
        assert_eq!(calculate_aspect_ratio(1000, 500), "2:1"); // Simplified custom ratio
        assert_eq!(calculate_aspect_ratio(1920, 1440), "4:3"); // Should match common ratio

        // Ultra-wide resolutions that don't match 21:9 exactly
        assert_eq!(calculate_aspect_ratio(2560, 1080), "64:27"); // 2.370, not exactly 21:9 (2.333)
        assert_eq!(calculate_aspect_ratio(3440, 1440), "43:18"); // 2.388, not exactly 21:9
    }

    #[test]
    fn test_calculate_aspect_ratio_edge_cases() {
        // Test edge cases
        assert_eq!(calculate_aspect_ratio(1, 1), "1:1"); // Square
        assert_eq!(calculate_aspect_ratio(1920, 1), "1920:1"); // Extreme ratio
    }

    #[test]
    fn test_gcd() {
        assert_eq!(gcd(1920, 1080), 120);
        assert_eq!(gcd(1280, 720), 80);
        assert_eq!(gcd(100, 50), 50);
        assert_eq!(gcd(17, 19), 1); // Coprime numbers
        assert_eq!(gcd(48, 18), 6);
    }

    #[test]
    fn test_gcd_commutative() {
        // GCD should be commutative: gcd(a, b) == gcd(b, a)
        assert_eq!(gcd(1920, 1080), gcd(1080, 1920));
        assert_eq!(gcd(100, 75), gcd(75, 100));
    }

    #[test]
    fn test_gcd_with_zero() {
        // gcd(n, 0) should equal n
        assert_eq!(gcd(42, 0), 42);
        assert_eq!(gcd(0, 42), 42);
    }
}
