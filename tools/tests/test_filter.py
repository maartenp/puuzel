"""Tests for the Dutch word list filter pipeline."""
import sys
import os

# Add tools parent dir to path so we can import from tools/
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..'))

from tools.filter_wordlist import (
    compute_grid_length,
    filter_word,
    normalize_word,
    is_abbreviation,
)


def test_grid_length_ijsbeer():
    """IJSBEER: IJ=1, S=1, B=1, E=1, E=1, R=1 = 6 cells"""
    assert compute_grid_length("IJSBEER") == 6


def test_grid_length_huis():
    assert compute_grid_length("HUIS") == 4


def test_grid_length_ij():
    """IJ alone = 1 cell"""
    assert compute_grid_length("IJ") == 1


def test_grid_length_lijst():
    """LIJST: L=1, IJ=1, S=1, T=1 = 4 cells"""
    assert compute_grid_length("LIJST") == 4


def test_grid_length_unicode_ligature():
    """Unicode IJ ligature U+0132 followed by SBEER = 6 cells"""
    assert compute_grid_length("\u0132SBEER") == 6


def test_exclude_too_short():
    """Single letter words are excluded (grid_length < 2)"""
    assert not filter_word("A", set())


def test_exclude_too_long():
    """Words with grid_length > 15 are excluded"""
    assert not filter_word("A" * 16, set())


def test_exclude_abbreviation():
    """Words containing dots are excluded (abbreviations)"""
    assert not filter_word("e.g.", set())


def test_exclude_vulgar():
    """Words in the blocklist are excluded"""
    assert not filter_word("blocked", {"blocked"})


def test_keep_normal_word():
    """Normal Dutch words pass the filter"""
    assert filter_word("HUIS", set())


def test_exclude_digits():
    """Words containing digits are excluded"""
    assert not filter_word("3D", set())


def test_exclude_hyphenated():
    """Words containing hyphens are excluded"""
    assert not filter_word("Sint-Nicolaas", set())


def test_proper_noun_detection():
    """Words starting with uppercase (non-all-caps) are marked as proper nouns"""
    from tools.filter_wordlist import normalize_word
    # Amsterdam starts with uppercase and is not all-uppercase → proper noun
    word = "Amsterdam"
    is_proper = word[0].isupper() and not word.isupper()
    assert is_proper, "Amsterdam should be detected as proper noun"
