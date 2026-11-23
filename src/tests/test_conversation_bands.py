"""Coverage goals must respect allowed bands, speaker and exact grammatical form."""

import contextlib
import io
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
from check_conversation_bands import validate


class BandCoverageTests(unittest.TestCase):
    def setUp(self):
        self.vocabulary = dict.fromkeys(["čaj", "kruh", "voda", "sok", "partner", "vode"])
        self.library = {"conversations": [
            {"id": str(band), "band": band, "exchanges": [
                {"partner": {"hr": "Voda?"}, "learner": {"hr": text}}
            ]}
            for band, text in enumerate(["ČAJ!", "Kruh.", "Voda.", "Sok."], 1)
        ]}
        self.bands = {
            "conversations": {str(band): band for band in range(1, 5)},
            "bands": [
                {"band": band, "conversation_count": 1, "cumulative_conversation_count": band,
                 "cumulative_learner_vocabulary_forms": band}
                for band in range(1, 5)
            ],
            "coverage_requirements": [
                {"band": 3, "include_earlier_bands": True, "first_vocabulary_forms": 3}
            ],
        }

    def check_coverage(self):
        with contextlib.redirect_stdout(io.StringIO()):
            validate(self.bands, self.library, self.vocabulary)

    def test_cumulative_coverage_includes_earlier_bands_and_lowercases(self):
        self.check_coverage()

    def test_runtime_band_must_match_the_reviewed_assignment(self):
        self.library["conversations"][0]["band"] = 4
        with self.assertRaisesRegex(ValueError, "Runtime band differs"):
            self.check_coverage()

    def test_independent_goal_cannot_borrow_from_earlier_bands(self):
        self.bands["coverage_requirements"][0]["include_earlier_bands"] = False
        with self.assertRaisesRegex(ValueError, "Band 3 learner replies are missing: čaj, kruh"):
            self.check_coverage()

    def test_cumulative_goal_cannot_borrow_from_later_bands(self):
        self.bands["coverage_requirements"][0]["first_vocabulary_forms"] = 4
        with self.assertRaisesRegex(ValueError, "missing: sok"):
            self.check_coverage()

    def test_partner_text_and_different_inflections_cannot_fill_a_gap(self):
        for learner_text in ["Partner.", "Vode."]:
            with self.subTest(learner_text=learner_text):
                self.library["conversations"][2]["exchanges"][0]["learner"]["hr"] = learner_text
                with self.assertRaisesRegex(ValueError, "missing: voda"):
                    self.check_coverage()


if __name__ == "__main__":
    unittest.main()
