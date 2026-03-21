"""
Claude Code CLI clue generator for Puuzel crossword generator.

Generates one descriptive Dutch crossword clue per word, plus a commonness
score (1-5) that determines word difficulty:
  - commonness 4-5 → easy word
  - commonness 3   → medium word
  - commonness 1-2 → hard word

Uses the claude CLI (Max subscription — no API key required).
Includes a self-verification pass and incremental progress saving.

Usage:
    python tools/generate_clues.py [--start N] [--count M] [--no-gate]

Exit codes:
    0 = completed successfully
    1 = other error
    2 = stopped due to rate limit (partial results saved)
    3 = quality gate rejected by user
"""

import argparse
import json
import os
import subprocess
import sys
import time

# Number of words per claude CLI call
CHUNK_SIZE = 50


class RateLimitError(Exception):
    """Raised when the claude CLI reports a usage/rate limit error."""
    pass


def generate_clues_for_chunk(words: list) -> list:
    """
    Generate one clue per word for up to 50 words in a single claude CLI call.

    Difficulty is NOT a property of the clue — it's derived from how common
    the word is (commonness score). Each word gets one straightforward
    descriptive clue.

    Args:
        words: list of word dicts with keys: word, grid_length, is_proper_noun

    Returns:
        list of dicts: [{"word": "...", "clue": "...", "commonness": 3, "is_archaic": false}, ...]
        Returns None if the call fails (non-rate-limit error).

    Raises:
        RateLimitError: if the claude CLI reports a usage/rate limit.
    """
    word_list = ", ".join(f'"{w["word"]}"' for w in words)
    prompt = f"""Genereer EEN kruiswoordraadsel-aanwijzing in het Nederlands voor elk van deze woorden:
[{word_list}]

Regels per woord:
- Bij voorkeur EEN woord als aanwijzing: een synoniem, categorie of kernbeschrijving (bijv. lente→seizoen, zweep→strafwerktuig, bonthoed→hoofddeksel)
- Gebruik alleen meerdere woorden als een enkel woord niet duidelijk genoeg is (maximaal 4 woorden)
- De aanwijzing mag GEEN deel van het antwoordwoord bevatten (bijv. bij CLUBTROFEE mag je niet "trofee" of "club" gebruiken)
- Geen woordspeling, cryptische of dubbelzinnige aanwijzingen
- Geef een gewoonheidsscore: 1=zeldzaam/vakjargon, 2=ongewoon, 3=bekend, 4=veelgebruikt, 5=alledaags
- Geef is_archaic: true als het woord ouderwets of louter literair is, anders false
- Als is_archaic=true, begin de aanwijzing met "Ouderwets: " (bijv. "Ouderwets: vrouw van adel")

Antwoord ALLEEN als een JSON-array, geen andere tekst:
[{{"word": "WOORD", "clue": "Korte omschrijving", "commonness": 3, "is_archaic": false}}, ...]"""

    result = subprocess.run(
        ["claude", "-p", prompt, "--output-format", "json", "--model", "claude-haiku-4-5-20251001"],
        capture_output=True, text=True, timeout=120
    )

    if result.returncode != 0:
        stderr_lower = result.stderr.lower()
        if "rate limit" in stderr_lower or "usage limit" in stderr_lower:
            raise RateLimitError(result.stderr)
        print(f"  [WARN] claude CLI error (exit {result.returncode}): {result.stderr[:200]}", file=sys.stderr)
        return None

    try:
        response = json.loads(result.stdout)
        text = response.get("result", "")
        # The LLM response may have markdown code fences — strip them
        text = text.strip()
        if text.startswith("```"):
            lines = text.split("\n")
            text = "\n".join(lines[1:-1]) if lines[-1].strip() == "```" else "\n".join(lines[1:])
        clue_data = json.loads(text)
        return clue_data
    except (json.JSONDecodeError, KeyError) as e:
        print(f"  [WARN] Failed to parse claude response: {e}", file=sys.stderr)
        print(f"  [WARN] Raw output: {result.stdout[:500]}", file=sys.stderr)
        return None


def verify_clues_chunk(clues: list) -> list:
    """
    Verify up to 50 clues in a single claude CLI call.

    Ask the model to answer each clue — if the answer matches the target word,
    the clue is considered verified.

    Args:
        clues: [{"id": "IJSBEER", "clue": "Wit poolroofdier", "word": "IJSBEER"}, ...]

    Returns:
        [{"id": "IJSBEER", "answer": "IJSBEER"}, ...]

    Raises:
        RateLimitError: if the claude CLI reports a usage/rate limit.
    """
    clue_list = "\n".join(f'{i+1}. [{c["id"]}] {c["clue"]}' for i, c in enumerate(clues))
    prompt = f"""Los elk kruiswoordraadsel op. Geef ALLEEN het antwoord als een enkel Nederlands woord per aanwijzing.

{clue_list}

Antwoord ALLEEN als een JSON-array:
[{{"id": "...", "answer": "..."}}, ...]"""

    result = subprocess.run(
        ["claude", "-p", prompt, "--output-format", "json", "--model", "claude-haiku-4-5-20251001"],
        capture_output=True, text=True, timeout=120
    )

    if result.returncode != 0:
        stderr_lower = result.stderr.lower()
        if "rate limit" in stderr_lower or "usage limit" in stderr_lower:
            raise RateLimitError(result.stderr)
        print(f"  [WARN] claude CLI error during verification (exit {result.returncode}): {result.stderr[:200]}", file=sys.stderr)
        return []

    try:
        response = json.loads(result.stdout)
        text = response.get("result", "")
        text = text.strip()
        if text.startswith("```"):
            lines = text.split("\n")
            text = "\n".join(lines[1:-1]) if lines[-1].strip() == "```" else "\n".join(lines[1:])
        return json.loads(text)
    except (json.JSONDecodeError, KeyError) as e:
        print(f"  [WARN] Failed to parse verification response: {e}", file=sys.stderr)
        return []


def clue_contains_word_part(word: str, clue: str) -> bool:
    """Check if any word in the clue is contained in the answer or vice versa.

    Splits the clue into whitespace-delimited tokens and checks:
    1. Is any clue token (3+ chars) a substring of the answer? (catches "houten" → no, but "hout" in HOUTROT → yes)
    2. Is the full answer word in the clue?

    Uses word stems by stripping common Dutch suffixes (en, e, s, er, je, jes, te, ste, ing)
    to catch inflected forms like "houten" → "hout".
    """
    word_lower = word.lower()
    clue_lower = clue.lower()

    # Full word check
    if word_lower in clue_lower:
        return True

    # Strip common Dutch suffixes to get stems
    suffixes = ("sten", "jes", "ing", "ste", "en", "er", "je", "te", "ig", "e", "s")

    def stem(w):
        """Return word and its stemmed form."""
        for suf in suffixes:
            if w.endswith(suf) and len(w) - len(suf) >= 3:
                return w[:len(w) - len(suf)]
        return w

    for token in clue_lower.split():
        token_stem = stem(token)
        # Is the clue token (or its stem) found inside the answer word?
        if len(token_stem) >= 3 and token_stem in word_lower:
            return True
        if len(token) >= 3 and token in word_lower:
            return True

    return False


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


def build_result(word_info: dict, clue_dict: dict, verification_map: dict) -> dict:
    """
    Build the final result dict for one word combining generation and verification data.

    Args:
        word_info: from filtered_words.json (word, grid_length, is_proper_noun)
        clue_dict: from generate_clues_for_chunk (clue, commonness, is_archaic)
        verification_map: {word: bool} — True if verified

    Returns:
        {
          "word": "IJSBEER",
          "grid_length": 6,
          "commonness": 4,
          "is_proper_noun": false,
          "is_archaic": false,
          "clues": [
            {"difficulty": "easy", "text": "Wit poolroofdier", "verified": true}
          ]
        }
    """
    word = word_info["word"]
    commonness = clue_dict.get("commonness", 3)
    clue_text = clue_dict.get("clue", "")
    verified = verification_map.get(word, False)
    difficulty = commonness_to_difficulty(commonness)

    clues_out = []
    if clue_text:
        if clue_contains_word_part(word, clue_text):
            print(f"  [REJECT] {word}: clue contains word part → \"{clue_text}\"")
        else:
            clues_out.append({
                "difficulty": difficulty,
                "text": clue_text,
                "verified": verified,
            })

    return {
        "word": word,
        "grid_length": word_info["grid_length"],
        "commonness": commonness,
        "is_proper_noun": word_info.get("is_proper_noun", False),
        "is_archaic": clue_dict.get("is_archaic", False),
        "clues": clues_out,
    }


def process_chunk(words_chunk: list) -> list:
    """
    Process one chunk of up to CHUNK_SIZE words: generate clues, then verify them.

    Returns list of result dicts (one per word that was successfully processed).
    Returns None if rate limit was hit.
    """
    print(f"  Generating clues for {len(words_chunk)} words...", flush=True)
    clue_data = generate_clues_for_chunk(words_chunk)
    if clue_data is None:
        print("  [WARN] Generation failed for this chunk, skipping.")
        return []

    # Build a map from word -> clue dict
    clue_map = {}
    for item in clue_data:
        w = item.get("word", "").strip().upper()
        if w:
            clue_map[w] = item

    # Build clue items for verification (one clue per word)
    verification_inputs = []
    for word_info in words_chunk:
        w = word_info["word"]
        clue_dict = clue_map.get(w)
        if not clue_dict:
            # LLM dropped this word from the batch — skip
            print(f"  [SKIP] {w} not found in LLM response")
            continue
        text = clue_dict.get("clue", "")
        if text:
            verification_inputs.append({
                "id": w,
                "clue": text,
                "word": w,
            })

    # Verify in batches of CHUNK_SIZE
    verification_map = {}
    for i in range(0, len(verification_inputs), CHUNK_SIZE):
        verify_batch = verification_inputs[i:i + CHUNK_SIZE]
        print(f"  Verifying {len(verify_batch)} clues...", flush=True)
        answers = verify_clues_chunk(verify_batch)
        for ans in answers:
            word_id = ans.get("id", "").strip().upper()
            answer = ans.get("answer", "").strip().upper()
            verification_map[word_id] = (answer == word_id)

    # Build results
    results = []
    for word_info in words_chunk:
        w = word_info["word"]
        clue_dict = clue_map.get(w)
        if not clue_dict:
            continue
        result = build_result(word_info, clue_dict, verification_map)
        results.append(result)

    return results


def run_quality_gate(sample_results: list, output_dir: str) -> bool:
    """
    Save sample results and ask the user to review before continuing.

    Returns True if the user approves, False if rejected.
    """
    sample_path = os.path.join(output_dir, "quality_sample.json")
    os.makedirs(output_dir, exist_ok=True)
    with open(sample_path, 'w', encoding='utf-8') as f:
        json.dump(sample_results, f, ensure_ascii=False, indent=2)

    print("\n" + "="*60)
    print("QUALITY GATE — Review sample clues before continuing")
    print("="*60)

    display_count = min(10, len(sample_results))
    for item in sample_results[:display_count]:
        word = item["word"]
        commonness = item.get("commonness", "?")
        is_archaic = item.get("is_archaic", False)
        archaic_note = " [ARCHAIC]" if is_archaic else ""
        difficulty = commonness_to_difficulty(int(commonness)) if isinstance(commonness, int) else "?"
        print(f"\n  {word} (commonness={commonness} → {difficulty}){archaic_note}")
        for clue in item.get("clues", []):
            verified = "OK" if clue.get("verified") else "FAIL"
            print(f"    [{verified}] {clue['text']}")

    print(f"\nFull sample saved to: {sample_path}")
    print("\nReview the sample clues above. Continue? [y/n] ", end="", flush=True)

    try:
        answer = input().strip().lower()
    except (EOFError, KeyboardInterrupt):
        answer = "n"

    return answer == "y"


def main():
    parser = argparse.ArgumentParser(
        description="Generate Dutch crossword clues using Claude Code CLI"
    )
    parser.add_argument("--start", type=int, default=0,
                        help="Index to start from in filtered_words.json (default: 0)")
    parser.add_argument("--count", type=int, default=10000,
                        help="Number of words to process in this run (default: 10000)")
    parser.add_argument("--input", default="tools/output/filtered_words.json",
                        help="Path to filtered words JSON file")
    parser.add_argument("--output-dir", default="tools/output",
                        help="Directory for output files (default: tools/output)")
    parser.add_argument("--chunk-size", type=int, default=CHUNK_SIZE,
                        help=f"Words per claude CLI call (default: {CHUNK_SIZE})")
    parser.add_argument("--no-gate", action="store_true",
                        help="Skip the quality gate (for unattended runs after initial approval)")
    args = parser.parse_args()

    chunk_size = args.chunk_size

    # Load filtered words
    input_path = args.input
    if not os.path.exists(input_path):
        print(f"Error: input file not found: {input_path}", file=sys.stderr)
        print("Run 'python tools/filter_wordlist.py' first.", file=sys.stderr)
        sys.exit(1)

    with open(input_path, 'r', encoding='utf-8') as f:
        all_words = json.load(f)

    total = len(all_words)
    start = args.start
    end = min(start + args.count, total)
    words_to_process = all_words[start:end]

    print(f"Processing words {start}–{end-1} ({len(words_to_process)} words) from {total} total")

    output_dir = args.output_dir
    os.makedirs(output_dir, exist_ok=True)

    batch_output_path = os.path.join(output_dir, f"clues_batch_{start}_{end}.json")

    all_results = []
    save_counter = 0
    is_first_chunk = True

    chunks = [words_to_process[i:i+chunk_size] for i in range(0, len(words_to_process), chunk_size)]
    total_chunks = len(chunks)

    for chunk_idx, chunk in enumerate(chunks):
        print(f"\n[Chunk {chunk_idx+1}/{total_chunks}] words {start + chunk_idx*chunk_size} – {start + chunk_idx*chunk_size + len(chunk) - 1}")

        try:
            results = process_chunk(chunk)
        except RateLimitError as e:
            print(f"\nRate limit hit: {e}", file=sys.stderr)
            # Save partial progress
            if all_results:
                with open(batch_output_path, 'w', encoding='utf-8') as f:
                    json.dump(all_results, f, ensure_ascii=False, indent=2)
            next_start = start + chunk_idx * chunk_size
            processed = chunk_idx * chunk_size
            print(f"\nRate limit hit after {processed} words.")
            print(f"Re-run with --start {next_start} to continue.")
            _print_summary(all_results)
            sys.exit(2)

        if results is None:
            continue

        # Quality gate after first chunk
        if is_first_chunk and not args.no_gate and results:
            is_first_chunk = False
            approved = run_quality_gate(results, output_dir)
            if not approved:
                print("\nQuality rejected. Adjust prompts and retry.")
                sys.exit(3)
            print("\nQuality approved. Continuing with remaining batches...")
        else:
            is_first_chunk = False

        all_results.extend(results)
        save_counter += len(results)

        # Save incrementally every 500 words
        if save_counter >= 500:
            with open(batch_output_path, 'w', encoding='utf-8') as f:
                json.dump(all_results, f, ensure_ascii=False, indent=2)
            print(f"  [SAVE] Progress saved: {len(all_results)} words so far")
            save_counter = 0

    # Final save
    with open(batch_output_path, 'w', encoding='utf-8') as f:
        json.dump(all_results, f, ensure_ascii=False, indent=2)

    _print_summary(all_results)
    print(f"\nResults written to: {batch_output_path}")
    sys.exit(0)


def _print_summary(results: list):
    """Print a summary of generation and verification results."""
    total_words = len(results)
    total_clues = 0
    verified_clues = 0
    difficulty_counts = {"easy": 0, "medium": 0, "hard": 0}

    for item in results:
        commonness = item.get("commonness", 3)
        difficulty = commonness_to_difficulty(commonness)
        difficulty_counts[difficulty] += 1
        for clue in item.get("clues", []):
            total_clues += 1
            if clue.get("verified"):
                verified_clues += 1

    pct = (verified_clues / total_clues * 100) if total_clues > 0 else 0

    print(f"\n{'='*60}")
    print(f"SUMMARY")
    print(f"{'='*60}")
    print(f"Words processed: {total_words}")
    print(f"Clues generated: {total_clues}, verified: {verified_clues} ({pct:.1f}%)")
    print(f"Word difficulty distribution:")
    for d, count in difficulty_counts.items():
        print(f"  {d:6}: {count:5} words")
    print(f"{'='*60}")


if __name__ == "__main__":
    main()
