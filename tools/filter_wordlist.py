"""
Dutch word list filter pipeline for Puuzel crossword generator.

Reads local dutch.txt from the repo root, filters for crossword-suitable words,
computes IJ-aware grid lengths, detects proper nouns, and outputs filtered_words.json.

Usage:
    python tools/filter_wordlist.py

Output:
    tools/output/filtered_words.json
"""

import json
import os
import sys


def normalize_word(word: str) -> str:
    """
    Uppercase the word and normalize Unicode IJ ligatures.

    Replaces:
    - U+0132 (LATIN CAPITAL LIGATURE IJ) → "IJ"
    - U+0133 (LATIN SMALL LIGATURE IJ) → "ij"

    Then converts to uppercase.
    """
    # Normalize IJ ligatures before uppercasing
    word = word.replace("\u0132", "IJ").replace("\u0133", "ij")
    return word.upper()


def compute_grid_length(word: str) -> int:
    """
    Compute the number of grid cells a word occupies.

    In Dutch crosswords, "IJ" is a digraph that occupies a single cell.
    This function counts tokens: "IJ" = 1 token, all other characters = 1 token.

    Examples:
        compute_grid_length("IJSBEER") == 6  (IJ-S-B-E-E-R)
        compute_grid_length("IJ") == 1
        compute_grid_length("LIJST") == 4  (L-IJ-S-T)
        compute_grid_length("HUIS") == 4
    """
    normalized = normalize_word(word)
    count = 0
    i = 0
    while i < len(normalized):
        if normalized[i:i+2] == "IJ":
            count += 1
            i += 2
        else:
            count += 1
            i += 1
    return count


def is_abbreviation(word: str) -> bool:
    """
    Returns True if the word is an abbreviation.

    Abbreviations are identified by the presence of a dot in the word (per D-07).
    """
    return '.' in word


def filter_word(word: str, blocklist: set) -> bool:
    """
    Returns True if the word should be KEPT (passes all filters).

    Filters applied:
    - grid_length must be between 2 and 15 inclusive (D-09, D-10)
    - No dots (abbreviations, D-07)
    - Not in the vulgarity blocklist (D-11)
    - No digits
    - No spaces or hyphens (compound separators; D-13 allows compounds without separators)
    """
    # Check for abbreviations (dot-containing words)
    if is_abbreviation(word):
        return False

    # Check for digits
    if any(c.isdigit() for c in word):
        return False

    # Check for spaces or hyphens
    if ' ' in word or '-' in word:
        return False

    # Check blocklist (case-insensitive)
    if word.lower() in blocklist:
        return False

    # Check grid length bounds
    grid_len = compute_grid_length(word)
    if grid_len < 2 or grid_len > 15:
        return False

    return True


def load_blocklist(blocklist_path: str) -> set:
    """Load the vulgarity blocklist into a set of lowercase words."""
    blocklist = set()
    if not os.path.exists(blocklist_path):
        print(f"Warning: blocklist not found at {blocklist_path}", file=sys.stderr)
        return blocklist
    with open(blocklist_path, 'r', encoding='utf-8') as f:
        for line in f:
            word = line.strip().lower()
            if word and not word.startswith('#'):
                blocklist.add(word)
    return blocklist


def process_wordlist(dutch_txt_path: str, blocklist_path: str) -> list:
    """
    Read dutch.txt, apply all filters, and return list of word dicts.

    Returns:
        list of {"word": str, "grid_length": int, "is_proper_noun": bool}
    """
    blocklist = load_blocklist(blocklist_path)

    results = []
    total_read = 0
    proper_noun_count = 0

    with open(dutch_txt_path, 'r', encoding='utf-8') as f:
        for line in f:
            word = line.strip()
            if not word:
                continue
            total_read += 1

            if not filter_word(word, blocklist):
                continue

            # Detect proper nouns: starts with uppercase, is not all-uppercase
            # All-uppercase could be an acronym like "TV" or "PC" — we allow those
            is_proper_noun = word[0].isupper() and not word.isupper()

            if is_proper_noun:
                proper_noun_count += 1

            # Normalize to uppercase for storage
            normalized = normalize_word(word)
            grid_len = compute_grid_length(normalized)

            results.append({
                "word": normalized,
                "grid_length": grid_len,
                "is_proper_noun": is_proper_noun,
            })

    return results, total_read, proper_noun_count


def main():
    """Run the filter pipeline and write output to tools/output/filtered_words.json."""
    # Paths relative to repo root
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    dutch_txt_path = os.path.join(repo_root, "dutch.txt")
    blocklist_path = os.path.join(repo_root, "tools", "dutch_blocklist.txt")
    output_dir = os.path.join(repo_root, "tools", "output")
    output_path = os.path.join(output_dir, "filtered_words.json")

    if not os.path.exists(dutch_txt_path):
        print(f"Error: dutch.txt not found at {dutch_txt_path}", file=sys.stderr)
        sys.exit(1)

    print(f"Reading Dutch word list from {dutch_txt_path}...")
    results, total_read, proper_noun_count = process_wordlist(dutch_txt_path, blocklist_path)

    os.makedirs(output_dir, exist_ok=True)
    with open(output_path, 'w', encoding='utf-8') as f:
        json.dump(results, f, ensure_ascii=False, indent=2)

    print(f"Total words read:    {total_read:,}")
    print(f"Words after filter:  {len(results):,}")
    print(f"Proper nouns:        {proper_noun_count:,}")
    print(f"Output written to:   {output_path}")


if __name__ == "__main__":
    main()
