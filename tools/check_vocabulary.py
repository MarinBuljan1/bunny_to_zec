"""Check vocabulary, explanation grouping and persistent score corrections.

Run: python tools/check_vocabulary.py
These checks catch data inconsistencies; they do not certify natural language.
"""

import json
from pathlib import Path
import unicodedata

ROOT = Path(__file__).resolve().parent.parent


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"Duplicate JSON key: {key!r}")
        result[key] = value
    return result


def load_json(path):
    return json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=unique_object)


def validate(vocabulary, explanations, manifest):
    if not vocabulary:
        raise ValueError("Vocabulary must not be empty")
    notes = set()
    for word, entry in vocabulary.items():
        if not word or word != word.strip().lower() or any(c.isspace() for c in word):
            raise ValueError(f"Expected a lowercase single-token vocabulary form: {word!r}")
        if unicodedata.normalize("NFC", word) != word:
            raise ValueError(f"Vocabulary form is not NFC-normalized: {word!r}")
        if any("CYRILLIC" in unicodedata.name(c, "") for c in word):
            raise ValueError(f"Unexpected Cyrillic character in Croatian form: {word!r}")
        if not isinstance(entry.get("base_language"), str) or not entry["base_language"].strip():
            raise ValueError(f"Missing English hint for {word!r}")
    for hint, group in explanations.items():
        if not group:
            raise ValueError(f"Empty explanation group: {hint!r}")
        for word, note in group.items():
            if word not in vocabulary:
                raise ValueError(f"Explanation has no vocabulary form: {word!r}")
            if vocabulary[word]["base_language"] != hint:
                raise ValueError(f"Explanation is in the wrong English group: {word!r}")
            if word in notes:
                raise ValueError(f"Explanation repeated across groups: {word!r}")
            if not isinstance(note, str) or not note.strip() or "REVIEW REQUIRED" in note:
                raise ValueError(f"Missing/unfinished explanation: {word!r}")
            notes.add(word)
    missing = vocabulary.keys() - notes
    if missing:
        raise ValueError(f"Missing explanations: {sorted(missing)}")
    if manifest.get("schema_version") != 1:
        raise ValueError("Unsupported correction manifest version")
    originals = set()
    for correction in manifest["corrections"]:
        old, new = correction["original"], correction["corrected"]
        if old in originals or old in vocabulary or new not in vocabulary:
            raise ValueError(f"Invalid or repeated score alias: {old!r} -> {new!r}")
        originals.add(old)
    for removal in manifest.get("split_phrases", []):
        if removal["original"] in vocabulary or removal["original"] in originals:
            raise ValueError(f"Split phrase cannot be a vocabulary word or score alias: {removal!r}")
        if len(removal["corrected"].split()) < 2:
            raise ValueError(f"Expected a multiword correction: {removal!r}")


def main():
    vocabulary = load_json(ROOT / "assets/croatian.json")
    explanations = load_json(ROOT / "assets/croatian_explanations.json")
    manifest = load_json(ROOT / "assets/vocabulary_corrections.json")
    validate(vocabulary, explanations, manifest)
    print(
        f"{len(vocabulary):,} vocabulary forms; one correctly grouped explanation each; "
        f"{len(manifest['corrections'])} valid progress aliases"
    )


if __name__ == "__main__":
    main()
