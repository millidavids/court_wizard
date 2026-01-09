/// Helper function to parse aspect ratio string to decimal value
pub(super) fn parse_aspect_ratio(ratio: &str) -> f32 {
    match ratio {
        "16:9" => 16.0 / 9.0,
        "16:10" => 16.0 / 10.0,
        "4:3" => 4.0 / 3.0,
        "21:9" => 21.0 / 9.0,
        _ => {
            // Try to parse custom ratio
            if let Some((w, h)) = ratio.split_once(':')
                && let (Ok(width), Ok(height)) = (w.parse::<f32>(), h.parse::<f32>())
            {
                return width / height;
            }
            16.0 / 9.0 // default
        }
    }
}

/// Helper function to get next aspect ratio in the cycle
pub(super) fn next_aspect_ratio(current: &str) -> &'static str {
    match current {
        "16:9" => "16:10",
        "16:10" => "4:3",
        "4:3" => "21:9",
        _ => "16:9", // default back to 16:9
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_aspect_ratio_common_ratios() {
        // Test parsing of common aspect ratio strings
        assert_eq!(parse_aspect_ratio("16:9"), 16.0 / 9.0);
        assert_eq!(parse_aspect_ratio("16:10"), 16.0 / 10.0);
        assert_eq!(parse_aspect_ratio("4:3"), 4.0 / 3.0);
        assert_eq!(parse_aspect_ratio("21:9"), 21.0 / 9.0);
    }

    #[test]
    fn test_parse_aspect_ratio_custom_ratios() {
        // Test parsing of custom ratio strings
        assert_eq!(parse_aspect_ratio("2:1"), 2.0);
        assert_eq!(parse_aspect_ratio("1:1"), 1.0);
        assert_eq!(parse_aspect_ratio("32:9"), 32.0 / 9.0);
    }

    #[test]
    fn test_parse_aspect_ratio_invalid_input() {
        // Test that invalid inputs fall back to default (16:9)
        assert_eq!(parse_aspect_ratio("invalid"), 16.0 / 9.0);
        assert_eq!(parse_aspect_ratio("16/9"), 16.0 / 9.0);
        assert_eq!(parse_aspect_ratio(""), 16.0 / 9.0);
        assert_eq!(parse_aspect_ratio("abc:def"), 16.0 / 9.0);
    }

    #[test]
    fn test_next_aspect_ratio_cycle() {
        // Test the full cycle of aspect ratios
        assert_eq!(next_aspect_ratio("16:9"), "16:10");
        assert_eq!(next_aspect_ratio("16:10"), "4:3");
        assert_eq!(next_aspect_ratio("4:3"), "21:9");
        assert_eq!(next_aspect_ratio("21:9"), "16:9"); // Wraps back
    }

    #[test]
    fn test_next_aspect_ratio_unknown() {
        // Test that unknown ratios default to 16:9
        assert_eq!(next_aspect_ratio("unknown"), "16:9");
        assert_eq!(next_aspect_ratio("2:1"), "16:9");
        assert_eq!(next_aspect_ratio(""), "16:9");
    }

    #[test]
    fn test_parse_and_next_consistency() {
        // Verify that all ratios in the cycle can be parsed
        let ratios = ["16:9", "16:10", "4:3", "21:9"];

        for ratio in ratios {
            let parsed = parse_aspect_ratio(ratio);
            assert!(
                parsed > 0.0,
                "Ratio {} should parse to positive value",
                ratio
            );

            let next = next_aspect_ratio(ratio);
            let next_parsed = parse_aspect_ratio(next);
            assert!(
                next_parsed > 0.0,
                "Next ratio {} should parse to positive value",
                next
            );
        }
    }
}
