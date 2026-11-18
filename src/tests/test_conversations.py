"""Content/index checks; these do not certify Croatian grammar or translation."""

from pathlib import Path
import sys
import unittest

TOOLS = Path(__file__).resolve().parents[2] / "tools"
sys.path.insert(0, str(TOOLS))
from check_conversations import inspect, tokens


def library():
    conversations = []
    for number in range(100):
        conversations.append({
            "id": f"{number + 1:04d}", "title": "Test", "topic": "test",
            "exchanges": [{"partner": {"hr": f"Pitanje {number}-{turn}?", "en": "A question."},
                           "learner": {"hr": "Kavu, kavu! Kruh i kruha.", "en": "Coffee, coffee! Bread and a form of bread."}} for turn in range(6)]})
    return {"schema_version": 1, "source_language": "hr", "translation_language": "en", "conversations": conversations}


class ConversationTests(unittest.TestCase):
    def test_lowercase_keeps_inflections_and_accents_distinct(self):
        self.assertEqual(tokens("ČAJ, čaj! Kruh i kruha?"), ["čaj", "čaj", "kruh", "i", "kruha"])
        self.assertNotEqual(tokens("caj"), tokens("čaj"))

    def test_index_counts_repeated_learner_words_and_excludes_partner_words(self):
        index, report = inspect(library(), {"kavu": {}, "kruh": {}, "kruha": {}, "pitanje": {}})
        self.assertEqual(index["kavu"][0], {"c_id": "0001", "hits": 12})
        self.assertEqual(index["kruh"][0]["hits"], 6)
        self.assertEqual(report["missing_words"], ["pitanje"])
        self.assertFalse(report["coverage_complete"])

    def test_duplicate_ids_empty_translations_and_short_conversations_are_rejected(self):
        for defect in ["duplicate", "translation", "length"]:
            candidate = library()
            if defect == "duplicate":
                candidate["conversations"][1]["id"] = candidate["conversations"][0]["id"]
            elif defect == "translation":
                candidate["conversations"][0]["exchanges"][0]["learner"]["en"] = ""
            else:
                candidate["conversations"][0]["exchanges"].pop()
                candidate["conversations"][0]["exchanges"].pop()
            with self.assertRaises(ValueError):
                inspect(candidate, {"kavu": {}})

    def test_complete_coverage_comes_from_messages_not_declared_metadata(self):
        candidate = library()
        candidate["coverage"] = {"coverage_complete": True}
        _, report = inspect(candidate, {"kavu": {}, "tomorrow": {}})
        self.assertFalse(report["coverage_complete"])
        self.assertEqual(report["missing_words"], ["tomorrow"])
        _, complete = inspect(candidate, {"kavu": {}, "kruh": {}, "kruha": {}})
        self.assertTrue(complete["coverage_complete"])

    def test_numeric_ids_allow_gaps_reordering_and_more_than_four_digits(self):
        candidate = library()
        candidate["conversations"][0]["id"] = "10000"
        candidate["conversations"][1]["id"] = "9999"
        candidate["conversations"].reverse()
        index, _ = inspect(candidate, {"kavu": {}})
        identifiers = {entry["c_id"] for entry in index["kavu"]}
        self.assertIn("10000", identifiers)
        self.assertIn("9999", identifiers)
        self.assertNotIn("0001", identifiers)

    def test_noncanonical_and_nonnumeric_ids_are_rejected(self):
        for identifier in [None, 1, "0000", "1", "001", "00001", "-001", "1000a", "cafe-order-001", "１２３４"]:
            with self.subTest(identifier=identifier):
                candidate = library()
                candidate["conversations"][0]["id"] = identifier
                with self.assertRaisesRegex(ValueError, "Invalid conversation ID"):
                    inspect(candidate, {"kavu": {}})


if __name__ == "__main__":
    unittest.main()
