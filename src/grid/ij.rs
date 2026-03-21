use crate::grid::types::LetterToken;

/// Tokenize a Dutch word into letter tokens, treating the IJ digraph as a single token.
///
/// Rules:
/// - Unicode IJ ligature (U+0132 Ĳ) and its lowercase (U+0133 ĳ) are normalized to "IJ"
/// - Input is uppercased before tokenization
/// - The sequence "IJ" (two chars) is emitted as a single `LetterToken::IJ`
/// - All other characters are emitted as `LetterToken::Single(char)`
///
/// # Examples
/// ```
/// use puuzel::grid::ij::tokenize_dutch_word;
/// use puuzel::grid::types::LetterToken;
///
/// let tokens = tokenize_dutch_word("IJSBEER");
/// assert_eq!(tokens.len(), 6); // IJ S B E E R
/// assert_eq!(tokens[0], LetterToken::IJ);
/// ```
pub fn tokenize_dutch_word(word: &str) -> Vec<LetterToken> {
    // Normalize Unicode IJ ligatures to two-char sequences
    let normalized = word
        .replace('\u{0132}', "IJ")  // Ĳ → IJ
        .replace('\u{0133}', "ij"); // ĳ → ij

    // Convert to uppercase
    let upper = normalized.to_uppercase();

    let chars: Vec<char> = upper.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == 'I' && i + 1 < chars.len() && chars[i + 1] == 'J' {
            tokens.push(LetterToken::IJ);
            i += 2;
        } else {
            tokens.push(LetterToken::Single(chars[i]));
            i += 1;
        }
    }

    tokens
}

/// Returns the grid length of a word — the number of cells it occupies.
/// IJ counts as one cell.
pub fn grid_length(word: &str) -> usize {
    tokenize_dutch_word(word).len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::types::LetterToken;

    #[test]
    fn test_ijsbeer_6_tokens() {
        let tokens = tokenize_dutch_word("IJSBEER");
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0], LetterToken::IJ);
        assert_eq!(tokens[1], LetterToken::Single('S'));
        assert_eq!(tokens[2], LetterToken::Single('B'));
        assert_eq!(tokens[3], LetterToken::Single('E'));
        assert_eq!(tokens[4], LetterToken::Single('E'));
        assert_eq!(tokens[5], LetterToken::Single('R'));
    }

    #[test]
    fn test_huis_no_false_ij_match() {
        let tokens = tokenize_dutch_word("HUIS");
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0], LetterToken::Single('H'));
        assert_eq!(tokens[1], LetterToken::Single('U'));
        assert_eq!(tokens[2], LetterToken::Single('I'));
        assert_eq!(tokens[3], LetterToken::Single('S'));
    }

    #[test]
    fn test_lijst_4_tokens() {
        let tokens = tokenize_dutch_word("LIJST");
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0], LetterToken::Single('L'));
        assert_eq!(tokens[1], LetterToken::IJ);
        assert_eq!(tokens[2], LetterToken::Single('S'));
        assert_eq!(tokens[3], LetterToken::Single('T'));
    }

    #[test]
    fn test_unicode_ligature_u0132() {
        // Ĳ (U+0132) should normalize to IJ token
        let word = "\u{0132}SBEER";
        let tokens = tokenize_dutch_word(word);
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0], LetterToken::IJ);
    }

    #[test]
    fn test_unicode_ligature_u0133_lowercase() {
        // ĳ (U+0133) lowercase ligature should normalize to IJ token
        let word = "\u{0133}sbeer";
        let tokens = tokenize_dutch_word(word);
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0], LetterToken::IJ);
    }

    #[test]
    fn test_lowercase_input_normalizes() {
        // "ijs" should normalize to uppercase and produce [IJ, S]
        let tokens = tokenize_dutch_word("ijs");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], LetterToken::IJ);
        assert_eq!(tokens[1], LetterToken::Single('S'));
    }

    #[test]
    fn test_grid_length_ijsbeer() {
        assert_eq!(grid_length("IJSBEER"), 6);
    }

    #[test]
    fn test_grid_length_lijst() {
        assert_eq!(grid_length("LIJST"), 4);
    }

    #[test]
    fn test_grid_length_huis() {
        assert_eq!(grid_length("HUIS"), 4);
    }

    #[test]
    fn test_no_ij_at_end_of_word() {
        // Word ending in 'I' with no following 'J' — should be Single('I')
        let tokens = tokenize_dutch_word("TAXI");
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[3], LetterToken::Single('I'));
    }

    #[test]
    fn test_multiple_ij_in_word() {
        // Artificial test: IJIJ should be [IJ, IJ]
        let tokens = tokenize_dutch_word("IJIJ");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], LetterToken::IJ);
        assert_eq!(tokens[1], LetterToken::IJ);
    }
}
