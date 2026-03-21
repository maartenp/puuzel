#!/usr/bin/env bash
# Batch runner for Dutch crossword clue generation.
# Processes the full filtered word list in batches of BATCH_SIZE words.
# Handles usage limits with automatic retry and supports resume from last batch.
#
# Usage:
#   ./tools/generate_clues_batch.sh
#
# Environment:
#   BATCH_SIZE  — words per batch (default: 10000)

set -euo pipefail

BATCH_SIZE=${BATCH_SIZE:-10000}
INPUT_FILE="tools/output/filtered_words.json"
OUTPUT_DIR="tools/output"
MERGED_FILE="$OUTPUT_DIR/verified_clues.json"

# Count total words
TOTAL=$(python -c "import json; print(len(json.load(open('$INPUT_FILE'))))")
echo "Total words to process: $TOTAL"

START=0

# Check for existing progress — find the highest batch file and resume from there
LATEST_BATCH=$(ls "$OUTPUT_DIR"/clues_batch_*.json 2>/dev/null | sort -t_ -k3 -n | tail -1 || true)
if [ -n "$LATEST_BATCH" ]; then
    # Extract the end index from filename like clues_batch_0_10000.json
    START=$(echo "$LATEST_BATCH" | grep -oP '\d+(?=\.json$)' || echo "0")
    echo "Resuming from word $START (found $LATEST_BATCH)"
fi

GATE_FLAG=""

while [ "$START" -lt "$TOTAL" ]; do
    echo ""
    echo "=== Processing batch: words $START to $((START + BATCH_SIZE)) ==="
    echo ""

    set +e
    python tools/generate_clues.py --start "$START" --count "$BATCH_SIZE" $GATE_FLAG
    EXIT_CODE=$?
    set -e

    if [ $EXIT_CODE -eq 3 ]; then
        echo ""
        echo "Quality rejected. Adjust prompts and retry."
        exit 1
    elif [ $EXIT_CODE -eq 2 ]; then
        echo ""
        echo "Rate limit hit. Checking reset time..."
        echo "Waiting 60 minutes for usage limit reset..."
        sleep 3600
        # Don't increment START — retry the same batch (it saved partial progress,
        # but we restart the batch to keep it simple)
        continue
    elif [ $EXIT_CODE -ne 0 ]; then
        echo "Error during generation. Check logs above."
        exit 1
    fi

    # After first successful batch, skip quality gate for subsequent batches
    GATE_FLAG="--no-gate"
    START=$((START + BATCH_SIZE))
done

echo ""
echo "=== All batches complete. Merging results... ==="

# Merge all batch files into a single verified_clues.json
python -c "
import json, glob
all_words = []
for f in sorted(glob.glob('$OUTPUT_DIR/clues_batch_*.json')):
    all_words.extend(json.load(open(f)))
# Deduplicate by word (in case of partial retries)
seen = set()
unique = []
for w in all_words:
    if w['word'] not in seen:
        seen.add(w['word'])
        unique.append(w)
with open('$MERGED_FILE', 'w') as f:
    json.dump(unique, f, ensure_ascii=False, indent=2)
print(f'Merged {len(unique)} words into $MERGED_FILE')
"

echo ""
echo "=== Done! Run 'python tools/write_database.py' to build puuzel.db ==="
