#!/usr/bin/env bash
# Batch runner for Dutch crossword clue generation.
# Processes the full filtered word list in parallel batches.
# Handles usage limits with automatic retry and supports resume.
#
# Usage:
#   ./tools/generate_clues_batch.sh
#
# Environment:
#   BATCH_SIZE  — words per batch (default: 10000)
#   PARALLEL    — number of parallel workers (default: 10)

set -euo pipefail

BATCH_SIZE=${BATCH_SIZE:-10000}
PARALLEL=${PARALLEL:-10}
INPUT_FILE="tools/output/filtered_words.json"
OUTPUT_DIR="tools/output"
MERGED_FILE="$OUTPUT_DIR/verified_clues.json"

# Count total words
TOTAL=$(python -c "import json; print(len(json.load(open('$INPUT_FILE'))))")
echo "Total words to process: $TOTAL"
echo "Parallel workers: $PARALLEL"
echo "Batch size: $BATCH_SIZE"

# Build list of batches that still need processing
BATCHES_TO_RUN=()
START=0
while [ "$START" -lt "$TOTAL" ]; do
    END=$((START + BATCH_SIZE))
    if [ "$END" -gt "$TOTAL" ]; then END=$TOTAL; fi
    BATCH_FILE="$OUTPUT_DIR/clues_batch_${START}_${END}.json"
    if [ -f "$BATCH_FILE" ]; then
        echo "  [SKIP] Batch $START-$END already exists"
    else
        BATCHES_TO_RUN+=("$START")
    fi
    START=$END
done

REMAINING=${#BATCHES_TO_RUN[@]}
echo ""
echo "Batches remaining: $REMAINING"

if [ "$REMAINING" -eq 0 ]; then
    echo "All batches complete!"
else
    # Quality gate on first batch if no batches exist yet
    EXISTING_COUNT=$(ls "$OUTPUT_DIR"/clues_batch_*.json 2>/dev/null | wc -l || echo "0")
    GATE_FLAG=""
    if [ "$EXISTING_COUNT" -eq 0 ]; then
        # Run first batch alone for quality gate review
        FIRST_START=${BATCHES_TO_RUN[0]}
        echo ""
        echo "=== Quality gate: running first batch ($FIRST_START) for review ==="
        echo ""
        set +e
        python tools/generate_clues.py --start "$FIRST_START" --count "$BATCH_SIZE"
        EXIT_CODE=$?
        set -e
        if [ $EXIT_CODE -eq 3 ]; then
            echo "Quality rejected. Adjust prompts and retry."
            exit 1
        elif [ $EXIT_CODE -ne 0 ]; then
            echo "First batch failed (exit $EXIT_CODE). Check logs above."
            exit 1
        fi
        # Remove first batch from the queue
        BATCHES_TO_RUN=("${BATCHES_TO_RUN[@]:1}")
        REMAINING=${#BATCHES_TO_RUN[@]}
        echo ""
        echo "Quality approved. Launching $REMAINING remaining batches ($PARALLEL in parallel)..."
    fi

    # Process remaining batches in parallel waves
    while [ ${#BATCHES_TO_RUN[@]} -gt 0 ]; do
        # Take up to PARALLEL batches
        WAVE_SIZE=$PARALLEL
        if [ ${#BATCHES_TO_RUN[@]} -lt $WAVE_SIZE ]; then
            WAVE_SIZE=${#BATCHES_TO_RUN[@]}
        fi

        WAVE=("${BATCHES_TO_RUN[@]:0:$WAVE_SIZE}")
        BATCHES_TO_RUN=("${BATCHES_TO_RUN[@]:$WAVE_SIZE}" 2>/dev/null || true)

        echo ""
        echo "=== Launching wave of $WAVE_SIZE parallel batches ==="
        PIDS=()
        for BATCH_START in "${WAVE[@]}"; do
            echo "  Starting batch at word $BATCH_START..."
            python tools/generate_clues.py --start "$BATCH_START" --count "$BATCH_SIZE" --no-gate &
            PIDS+=($!)
        done

        # Wait for all and collect results
        FAILED=0
        RATE_LIMITED=0
        for i in "${!PIDS[@]}"; do
            PID=${PIDS[$i]}
            BATCH_START=${WAVE[$i]}
            set +e
            wait "$PID"
            EXIT_CODE=$?
            set -e
            if [ $EXIT_CODE -eq 2 ]; then
                echo "  [RATE LIMIT] Batch $BATCH_START hit rate limit"
                RATE_LIMITED=1
                # Re-queue this batch
                BATCHES_TO_RUN+=("$BATCH_START")
            elif [ $EXIT_CODE -ne 0 ]; then
                echo "  [FAILED] Batch $BATCH_START (exit $EXIT_CODE)"
                FAILED=$((FAILED + 1))
                # Re-queue for retry
                BATCHES_TO_RUN+=("$BATCH_START")
            else
                echo "  [DONE] Batch $BATCH_START"
            fi
        done

        # Update DB after each wave
        echo "Updating database with latest results..."
        python tools/write_database.py
        echo ""

        if [ $RATE_LIMITED -eq 1 ]; then
            echo "Rate limit hit. Waiting 60 minutes for reset..."
            sleep 3600
        fi
    done
fi

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
echo "=== Done! Database updated at data/puuzel.db ==="
