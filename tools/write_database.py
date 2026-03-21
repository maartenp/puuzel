"""
Database writer for Puuzel crossword generator.

Reads tools/output/verified_clues.json and writes to data/puuzel.db (SQLite).
Only inserts verified clues (verified=true).

Usage:
    python tools/write_database.py [--input PATH] [--db PATH]
"""

import argparse
import json
import os
import sqlite3
import sys


def create_database(db_path: str, conn: sqlite3.Connection = None) -> sqlite3.Connection:
    """
    Create the SQLite database schema.

    Schema matches the Rust db/schema.rs for the runtime game binary.

    Returns the sqlite3 connection.
    """
    close_after = False
    if conn is None:
        os.makedirs(os.path.dirname(db_path), exist_ok=True)
        conn = sqlite3.connect(db_path)
        close_after = False

    conn.executescript("""
        CREATE TABLE IF NOT EXISTS words (
            id INTEGER PRIMARY KEY,
            word TEXT NOT NULL UNIQUE,
            grid_length INTEGER NOT NULL,
            commonness_score INTEGER NOT NULL,
            is_proper_noun INTEGER NOT NULL DEFAULT 0,
            is_archaic INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS clues (
            id INTEGER PRIMARY KEY,
            word_id INTEGER NOT NULL REFERENCES words(id),
            difficulty TEXT NOT NULL CHECK(difficulty IN ('easy', 'medium', 'hard')),
            clue_text TEXT NOT NULL,
            verified INTEGER NOT NULL DEFAULT 0,
            thumbs_down INTEGER NOT NULL DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_words_grid_length ON words(grid_length);
        CREATE INDEX IF NOT EXISTS idx_words_commonness ON words(commonness_score);
        CREATE INDEX IF NOT EXISTS idx_clues_word_difficulty ON clues(word_id, difficulty);
        CREATE INDEX IF NOT EXISTS idx_clues_verified ON clues(verified);
    """)
    conn.commit()
    return conn


def commonness_to_difficulty(commonness: int) -> str:
    """Derive word difficulty from commonness score.

    Common/everyday words are easy, rare/specialist words are hard.
    """
    if commonness >= 4:
        return "easy"
    elif commonness == 3:
        return "medium"
    else:
        return "hard"


def insert_words(conn: sqlite3.Connection, verified_clues: list) -> tuple:
    """
    Insert all verified words and their clues into the database.

    Difficulty is derived from the word's commonness score, not the clue style.
    Only inserts clues where verified=True.

    Returns:
        (words_inserted, clues_by_difficulty) tuple
    """
    cursor = conn.cursor()
    words_inserted = 0
    clues_by_difficulty = {"easy": 0, "medium": 0, "hard": 0}
    archaic_count = 0

    for item in verified_clues:
        word = item.get("word", "").strip()
        if not word:
            continue

        grid_length = item.get("grid_length", 0)
        commonness = item.get("commonness", 3)
        is_proper_noun = 1 if item.get("is_proper_noun", False) else 0
        is_archaic = 1 if item.get("is_archaic", False) else 0

        # Clamp commonness to valid range 1-5
        commonness = max(1, min(5, int(commonness)))
        difficulty = commonness_to_difficulty(commonness)

        try:
            cursor.execute(
                """INSERT OR IGNORE INTO words (word, grid_length, commonness_score, is_proper_noun, is_archaic)
                   VALUES (?, ?, ?, ?, ?)""",
                (word, grid_length, commonness, is_proper_noun, is_archaic)
            )
        except sqlite3.IntegrityError:
            # Duplicate word — skip
            continue

        # Get the word_id (handles both new inserts and existing rows)
        cursor.execute("SELECT id FROM words WHERE word = ?", (word,))
        row = cursor.fetchone()
        if row is None:
            continue
        word_id = row[0]
        words_inserted += 1
        if is_archaic:
            archaic_count += 1

        # Insert clues
        for clue in item.get("clues", []):
            clue_text = clue.get("text", "").strip()
            verified = 1 if clue.get("verified", False) else 0

            if not clue_text:
                continue

            cursor.execute(
                """INSERT INTO clues (word_id, difficulty, clue_text, verified, thumbs_down)
                   VALUES (?, ?, ?, ?, 0)""",
                (word_id, difficulty, clue_text, verified)
            )
            clues_by_difficulty[difficulty] += 1

    conn.commit()
    return words_inserted, clues_by_difficulty, archaic_count


def run_validation_queries(conn: sqlite3.Connection):
    """Run and print validation queries after insertion."""
    cursor = conn.cursor()

    print("\n--- Validation Queries ---")

    cursor.execute("SELECT COUNT(*) FROM words")
    word_count = cursor.fetchone()[0]
    print(f"Total words: {word_count:,}")

    cursor.execute("SELECT difficulty, COUNT(*) FROM clues WHERE verified=1 GROUP BY difficulty ORDER BY difficulty")
    print("Verified clues per difficulty:")
    for row in cursor.fetchall():
        print(f"  {row[0]:6}: {row[1]:,}")

    cursor.execute("SELECT grid_length, COUNT(*) FROM words GROUP BY grid_length ORDER BY grid_length")
    print("Words by grid length:")
    for row in cursor.fetchall():
        print(f"  length {row[0]:2}: {row[1]:,}")

    cursor.execute("SELECT COUNT(*) FROM words WHERE is_archaic=1")
    archaic = cursor.fetchone()[0]
    print(f"Archaic words: {archaic:,}")


def get_db_size(db_path: str) -> str:
    """Return human-readable database file size."""
    size = os.path.getsize(db_path)
    if size < 1024 * 1024:
        return f"{size / 1024:.1f} KB"
    return f"{size / (1024 * 1024):.1f} MB"


def load_clue_files(input_path: str) -> list:
    """Load clues from a single file or all batch files in a directory.

    Supports:
    - A single JSON file (verified_clues.json or any batch file)
    - A directory containing clues_batch_*.json files
    - The default tools/output/ directory (auto-discovers batch files)

    Deduplicates by word name.
    """
    import glob

    files_to_load = []

    if os.path.isfile(input_path):
        files_to_load = [input_path]
    elif os.path.isdir(input_path):
        files_to_load = sorted(glob.glob(os.path.join(input_path, "clues_batch_*.json")))
        if not files_to_load:
            # Try verified_clues.json as fallback
            merged = os.path.join(input_path, "verified_clues.json")
            if os.path.exists(merged):
                files_to_load = [merged]
    else:
        return []

    all_words = []
    for f in files_to_load:
        print(f"  Loading {f}...")
        with open(f, 'r', encoding='utf-8') as fh:
            all_words.extend(json.load(fh))

    # Deduplicate by word
    seen = set()
    unique = []
    for w in all_words:
        word = w.get("word", "")
        if word and word not in seen:
            seen.add(word)
            unique.append(w)

    return unique


def main():
    parser = argparse.ArgumentParser(
        description="Write verified clues to SQLite database (data/puuzel.db)"
    )
    parser.add_argument("--input", default="tools/output",
                        help="Path to clue JSON file or directory with batch files (default: tools/output)")
    parser.add_argument("--db", default="data/puuzel.db",
                        help="Path to output SQLite database (default: data/puuzel.db)")
    args = parser.parse_args()

    input_path = args.input
    db_path = args.db

    verified_clues = load_clue_files(input_path)
    if not verified_clues:
        print(f"Error: no clue files found at: {input_path}", file=sys.stderr)
        print("Run './tools/generate_clues_batch.sh' first, or point --input to a batch file.", file=sys.stderr)
        sys.exit(1)

    print(f"Loaded {len(verified_clues):,} unique word records")

    print(f"\nCreating database at {db_path}...")
    os.makedirs(os.path.dirname(db_path) if os.path.dirname(db_path) else ".", exist_ok=True)
    conn = sqlite3.connect(db_path)
    create_database(db_path, conn)

    print("Inserting words and clues...")
    words_inserted, clues_by_difficulty, archaic_count = insert_words(conn, verified_clues)

    total_clues = sum(clues_by_difficulty.values())

    print(f"\n--- Insertion Summary ---")
    print(f"Words inserted:   {words_inserted:,}")
    print(f"Clues inserted:   {total_clues:,}")
    for d, count in clues_by_difficulty.items():
        print(f"  {d:6}: {count:,}")
    print(f"Archaic words:    {archaic_count:,}")
    print(f"Database size:    {get_db_size(db_path)}")

    run_validation_queries(conn)
    conn.close()

    print(f"\nDatabase written to: {db_path}")


if __name__ == "__main__":
    main()
