"""Validate bilingual conversation drafts and measure exact learner-only coverage.

Run with --write-index after editing the JSON to refresh its selection index.
Run with --require-full-coverage to fail if any vocabulary entry is uncovered.
Neither command changes the vocabulary or the app's saved progress.
"""

import argparse
from collections import Counter, defaultdict
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parent.parent
LIBRARY = ROOT / "assets" / "conversations.json"
VOCABULARY = ROOT / "assets" / "croatian.json"


def tokens(text):
    # Match the current Rust separators; lowercase only, without stemming or
    # removing Croatian diacritics. Reviewed misspellings are fixed in the source.
    return [token.lower() for token in re.split(r"[ ,.!?|]", text) if token]


def inspect(library, vocabulary):
    if not isinstance(library, dict):
        raise ValueError("Conversation library must be an object")
    if library.get("schema_version") != 2:
        raise ValueError("Unsupported conversation schema version")
    if library.get("source_language") != "hr" or library.get("translation_language") != "en":
        raise ValueError("Expected Croatian messages and English translations")
    conversations = library.get("conversations")
    if not isinstance(conversations, list) or len(conversations) < 100:
        raise ValueError("The library must contain at least 100 conversations")
    keys = {key.lower() for key in vocabulary}
    identifiers = set()
    signatures = set()
    word_index = defaultdict(list)
    unlisted = Counter()
    for conversation in conversations:
        if not isinstance(conversation, dict):
            raise ValueError("Each conversation must be an object")
        identifier = conversation.get("id")
        # Canonical positive numbers: 0001 through 9999, then 10000 and up.
        # Gaps/reordering are accepted here; allocation is handled when authoring.
        if (
            not isinstance(identifier, str)
            or not re.fullmatch(r"(?:[0-9]{4}|[1-9][0-9]{4,})", identifier)
            or identifier == "0000"
        ):
            raise ValueError(f"Invalid conversation ID: {identifier!r}")
        if identifier in identifiers:
            raise ValueError(f"Duplicate conversation ID: {identifier}")
        identifiers.add(identifier)
        if type(conversation.get("band")) is not int or conversation["band"] not in range(1, 5):
            raise ValueError(f"Invalid conversation difficulty band in {identifier}")
        if not all(isinstance(conversation.get(field), str) and conversation[field].strip() for field in ["title", "topic"]):
            raise ValueError(f"Missing title/topic in {identifier}")
        exchanges = conversation.get("exchanges")
        if not isinstance(exchanges, list) or not 5 <= len(exchanges) <= 8:
            raise ValueError(f"Expected 5-8 exchanges in {identifier}")
        learner_counts = Counter()
        signature = []
        for exchange in exchanges:
            if not isinstance(exchange, dict):
                raise ValueError(f"Invalid exchange in {identifier}")
            for speaker in ["partner", "learner"]:
                message = exchange.get(speaker)
                if not isinstance(message, dict):
                    raise ValueError(f"Missing {speaker} message in {identifier}")
                for language in ["hr", "en"]:
                    text = message.get(language)
                    if not isinstance(text, str) or not text.strip() or len(text) > 1000:
                        raise ValueError(f"Invalid {speaker}/{language} text in {identifier}")
                signature.append(message["hr"])
            learner_counts.update(tokens(exchange["learner"]["hr"]))
        signature = tuple(signature)
        if signature in signatures:
            raise ValueError(f"Duplicate complete conversation: {identifier}")
        signatures.add(signature)
        for word, count in learner_counts.items():
            if word in keys:
                word_index[word].append({"c_id": identifier, "hits": count})
            else:
                unlisted[word] += count
    missing = sorted(keys - word_index.keys())
    return dict(sorted(word_index.items())), {
        "conversation_count": len(conversations),
        "vocabulary_count": len(keys),
        "covered_vocabulary_count": len(keys) - len(missing),
        "missing_vocabulary_count": len(missing),
        "coverage_complete": not missing,
        "missing_words": missing,
        "unlisted_learner_tokens": dict(sorted(unlisted.items())),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write-index", action="store_true")
    parser.add_argument("--require-full-coverage", action="store_true")
    arguments = parser.parse_args()
    library = json.loads(LIBRARY.read_text(encoding="utf-8"))
    vocabulary = json.loads(VOCABULARY.read_text(encoding="utf-8"))
    index, report = inspect(library, vocabulary)
    if arguments.write_index:
        library["word_index"] = index
        library["coverage"] = {key: value for key, value in report.items() if key not in ["missing_words", "unlisted_learner_tokens"]}
        LIBRARY.write_text(json.dumps(library, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    elif library.get("word_index") != index:
        raise ValueError("Conversation index is missing/stale; run with --write-index")
    elif library.get("coverage") != {key: value for key, value in report.items() if key not in ["missing_words", "unlisted_learner_tokens"]}:
        raise ValueError("Coverage summary is stale; run with --write-index")
    report_path = ROOT / "CONVERSATION_COVERAGE.json"
    report_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"{report['conversation_count']} conversations; {report['covered_vocabulary_count']}/{report['vocabulary_count']} entries covered; {report['missing_vocabulary_count']} missing")
    if arguments.require_full_coverage and report["missing_words"]:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
