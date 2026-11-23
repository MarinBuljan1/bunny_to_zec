"""Check editorial band assignments, summaries and learner-only coverage goals.

Bands remain review metadata; this check does not change runtime selection.
Vocabulary positions follow JSON key order, with exact lowercase token matching.
Run after check_conversations.py when changing conversations or their bands.
"""

from collections import Counter
import hashlib
import json
from pathlib import Path

from check_conversations import tokens

ROOT = Path(__file__).resolve().parent.parent


def learner_coverage_by_band(library, assignments):
    identifiers = [conversation["id"] for conversation in library["conversations"]]
    if len(set(identifiers)) != len(identifiers) or set(identifiers) != set(assignments):
        raise ValueError("Each conversation must have exactly one band assignment")
    if any(type(band) is not int or band not in range(1, 5) for band in assignments.values()):
        raise ValueError("Every band must be an integer from 1 to 4")
    coverage = {band: set() for band in range(1, 5)}
    for conversation in library["conversations"]:
        words = coverage[assignments[conversation["id"]]]
        for exchange in conversation["exchanges"]:
            words.update(tokens(exchange["learner"]["hr"]))
    return coverage


def validate(bands, library, vocabulary):
    assignments = bands["conversations"]
    coverage = learner_coverage_by_band(library, assignments)
    counts = Counter(assignments.values())
    summaries = bands["bands"]
    if [summary["band"] for summary in summaries] != [1, 2, 3, 4]:
        raise ValueError("Expected one summary for each band in order")
    cumulative_words = set()
    cumulative_count = 0
    for summary in summaries:
        band = summary["band"]
        cumulative_count += counts[band]
        cumulative_words.update(coverage[band])
        actual = (counts[band], cumulative_count, len(cumulative_words & vocabulary.keys()))
        recorded = tuple(summary[key] for key in (
            "conversation_count", "cumulative_conversation_count", "cumulative_learner_vocabulary_forms"
        ))
        if actual != recorded:
            raise ValueError(f"Band {band} summary is stale: expected {actual}, found {recorded}")
    for requirement in bands["coverage_requirements"]:
        band = requirement["band"]
        included_bands = range(1, band + 1) if requirement.get("include_earlier_bands", False) else (band,)
        eligible_words = set().union(*(coverage[number] for number in included_bands))
        eligible_count = sum(counts[number] for number in included_bands)
        label = f"Bands 1-{band}" if requirement.get("include_earlier_bands", False) else f"Band {band}"
        count = requirement["first_vocabulary_forms"]
        required_words = list(vocabulary)[:count]
        if len(required_words) != count:
            raise ValueError(f"Vocabulary has fewer than {count} forms")
        missing = [word for word in required_words if word not in eligible_words]
        if missing:
            raise ValueError(f"{label} learner replies are missing: {', '.join(missing)}")
        if eligible_count < requirement.get("minimum_conversations", 0):
            raise ValueError(f"{label} has too few conversations")
        print(f"{label}: {eligible_count} conversations; first {count}/{count} forms covered in learner replies")


def main():
    bands = json.loads((ROOT / "CONVERSATION_DIFFICULTY_BANDS.json").read_text(encoding="utf-8"))
    contents = []
    for name in ("source", "vocabulary"):
        path = ROOT / bands[f"{name}_file"]
        raw = path.read_bytes()
        if hashlib.sha256(raw).hexdigest() != bands[f"{name}_sha256"]:
            raise ValueError(f"{path.name} changed; review band assignments and refresh their summaries/hash")
        contents.append(json.loads(raw))
    validate(bands, *contents)


if __name__ == "__main__":
    main()
