"""Regression checks for editorial data boundaries (standard library only)."""

import copy
import unittest

from check_vocabulary import ROOT, load_json, unique_object, validate


class VocabularyValidationTests(unittest.TestCase):
    def setUp(self):
        self.words = {"bok": {"base_language": "hello"}}
        self.notes = {"hello": {"bok": "Informal greeting or farewell."}}
        self.manifest = {"schema_version": 1, "corrections": []}

    def test_shipped_assets_are_consistent(self):
        validate(
            load_json(ROOT / "assets/croatian.json"),
            load_json(ROOT / "assets/croatian_explanations.json"),
            load_json(ROOT / "assets/vocabulary_corrections.json"),
        )

    def test_duplicate_json_keys_are_rejected(self):
        with self.assertRaisesRegex(ValueError, "Duplicate"):
            unique_object([("bok", 1), ("bok", 2)])

    def test_changed_hint_requires_regrouping_its_note(self):
        self.words["bok"]["base_language"] = "hello; goodbye"
        with self.assertRaisesRegex(ValueError, "wrong English group"):
            validate(self.words, self.notes, self.manifest)

    def test_missing_and_orphan_notes_are_rejected(self):
        with self.assertRaisesRegex(ValueError, "Missing explanations"):
            validate(self.words, {}, self.manifest)
        notes = copy.deepcopy(self.notes)
        notes["hello"]["typo"] = "Unlisted form."
        with self.assertRaisesRegex(ValueError, "no vocabulary form"):
            validate(self.words, notes, self.manifest)

    def test_alias_must_end_at_a_current_word(self):
        self.manifest["corrections"] = [{"original": "old", "corrected": "absent"}]
        with self.assertRaisesRegex(ValueError, "score alias"):
            validate(self.words, self.notes, self.manifest)

    def test_split_phrase_is_not_assigned_to_a_single_word(self):
        self.manifest["split_phrases"] = [{"original": "joined", "corrected": "two words"}]
        validate(self.words, self.notes, self.manifest)
        self.manifest["corrections"] = [{"original": "joined", "corrected": "bok"}]
        with self.assertRaisesRegex(ValueError, "Split phrase"):
            validate(self.words, self.notes, self.manifest)


if __name__ == "__main__":
    unittest.main()
