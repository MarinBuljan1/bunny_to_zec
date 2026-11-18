# Croatian vocabulary review

Reviewed on 18 November 2026. The requested policy is **standard Croatian forms**,
with lowercase matching and distinct grammatical forms retained.

This completes a language-review pass, not a native-speaker certification. All
3,768 original translation families were inspected. Explanation patterns were
reviewed, inflection claims were cross-checked, and individual exceptions were
edited by hand. A dictionary or a morphological match cannot establish every
meaning a word may have in a particular sentence.

## Result

| Measure | Result |
| --- | ---: |
| Original vocabulary forms | 8,574 |
| Current vocabulary forms | 8,380 |
| New standard forms replacing former entries | 9 |
| Correction mappings added | 193 |
| Incorrectly joined phrases removed as single-word targets | 10 |
| English hints corrected beyond capitalization changes | 278 |
| English hints changed, including capitalization | 522 |
| Existing explanations revised | 4,319 |
| Previously missing explanations supplied for retained forms | 102 |
| Explanations supplied for new standard forms | 9 |
| Orphan explanation forms removed | 14 |
| Vocabulary forms without an explanation now | 0 |
| Conversation messages corrected | 9 |

The 193 replacements consist of 184 merges into existing forms and nine new
standard forms. Together with the ten split phrases, this accounts for the net
reduction of 194 vocabulary entries. Existing canonical entries retain their
relative order, which preserves their review-priority tie order.

There are still 1,000 conversations, covering **5,673 of 8,380 forms (67.7%)**.
Another 2,707 forms remain absent from learner replies. No conversations were
added or removed in this review.

## Important corrections

| Form | Correction or clarification |
| --- | --- |
| `se` | Reflexive pronoun, not “are.” |
| `sam` | Both “I am” and the masculine adjective “alone.” |
| `mi` | Both “we” and the short dative “to me.” |
| `nisam`, `nisi`, `nismo`, `niste` | Negative present forms of `biti`; a following participle creates a negative past construction. |
| `jeste` | Standard Croatian second-person plural “you are,” rather than third-person singular “is.” |
| `svoj` and its forms | Possession refers to the subject; it is not fixed to “my,” “his” or “their.” |
| `kupac` | Buyer/customer, not bather. |
| `koliki`, `kolika` | Questions about size or quantity, not colic. |
| `kolač` | Cake/pastry, not stake. |
| `stav` | Attitude/stance, not paragraph. |
| `desetak` | About ten, not a dozen. |
| `sad-a`, `sad-u` | Inflected forms of the abbreviation SAD (USA), not forms of the adverb “now.” |
| `spavaću` | An adjective form, as in `spavaću sobu`; “I will sleep” is `spavat ću`. |
| `daću` | A valid form of the noun `daća` (levy/tax); “I will give” is `dat ću`. |
| `sef` / `šef` | Safe / boss: both retained as different words. |
| `vise` / `više` | “They hang” / “more”: both retained. |
| `biće`, `biću` | Valid noun forms meaning being/creature; joined future constructions are not their standard Croatian use. |
| `Bille`, `Pam`, `Brad`, `Moore`, `Isaac`, and other names | Corrected unrelated English hints or notes that incorrectly ruled out the personal name. |

Examples of standardisation include `uvek` → `uvijek`, `vreme` → `vrijeme`,
`devojka` → `djevojka`, `spisak` → `popis`, and `obaveza` → `obveza`.
This is not accent stripping or stemming: forms such as `riječ` and `riječi`
remain separate targets. Likewise, valid standard alternatives such as `ponovo`
and `ponovno`, or `reci` and `kaži`, remain separate.

The full handwritten decisions and counts are in `VOCABULARY_REVIEW.json`.
The runtime spelling/standardisation mappings are in
`assets/vocabulary_corrections.json`; that manifest is also the source used by
saved-progress migration.

## Explanation changes

- Distinguish active past participles from a complete finite past tense, and
  identify passive participles where appropriate.
- Correct case/number errors, including `tjednu`, `tame`, `sati`, `damo`, and
  singular vocatives previously labelled plural.
- Where an explicitly identified noun lemma is supported, list the relevant
  singular/plural case forms and noun gender. These are possible analyses;
  the sentence determines which one applies.
- Explain shortened `-ti` infinitives before a future auxiliary, rather than
  labelling every shortened form a mistake or giving it a person/tense it lacks.
- Identify proper names, abbreviations, register, and vulgar or insulting uses.
- Supply missing notes and regroup every explanation under its current English
  hint, as required by the app's lookup.

The longest current note is 181 characters. Some notes are deliberately longer
than their predecessors because the former one-line definition omitted an
important grammatical distinction. Notes describe useful/common readings, not
an exhaustive dictionary entry for every homograph.

## Saved progress and conversations

The correction manifest now has 196 one-word mappings, including the three
historical corrections. When both spellings have progress, migration keeps the
most recent review; equal timestamps keep the higher score. Scores are never
added together. Valid homographs such as `sef`, `vise` and `biću` are not merged.

The ten split phrases, including `nemogu` → `ne mogu`, `samnom` → `sa mnom`,
and `bićeš` → `bit ćeš`, are no longer single-word targets. Their unmatched
historical score entries remain stored; transferring them to a constituent
would invent credit for another word. The progress schema version is unchanged.

Nine dialogue messages were corrected for `obaveza` and its inflections, or
`a-ha` → `aha`. Their English translations already expressed the intended
meaning. Conversation IDs, exchanges and scoring formulas are unchanged.
Conversation 0145 deliberately quotes regional `šta` while comparing it with
standard `što`; that example remains, but `šta` is no longer a vocabulary target.
The exact learner-only index and coverage report were regenerated.

## References and limits

Inflections were cross-checked with Nikola Ljubešić's
[hrLex 1.3 (2019), published through CLARIN.SI](https://www.clarin.si/repository/xmlui/handle/11356/1232),
licensed CC BY-SA 4.0. The reference archive and extracted analyses are local,
ignored working files, not additional runtime downloads. Its most frequent
analysis was not automatically accepted: several homographs and proper names
required a different interpretation. Notes were written for this app rather
than copied from dictionary prose.

The [Hrvatski jezični portal](https://hjp.znanje.hr/) was used for lexical and
inflectional checks. Its regional entries do not by themselves establish a
recommended standard form. The Institute's
[Jezični savjetnik](https://jezicni-savjetnik.hr/?page=21) supports standard choices
such as `popis` rather than `spisak`. The Institute's
[school grammar on indefinite pronouns](https://gramatika.hr/pravilo/neodredjene-zamjenice/31/)
distinguishes joined `štogod` from separate `što god`.

Language references sometimes differ in register or preferred alternatives.
Context-dependent meanings and stylistic naturalness remain appropriate subjects
for a native Croatian editor's review. Automated checks establish consistency,
not perfect Croatian or perfect translation in every future context.

No external AI API or additional model was used.

## Validation

- `python tools/check_vocabulary.py`: duplicate keys, missing/orphan/misgrouped
  notes, lowercase/NFC word forms, mixed Cyrillic spelling, and correction targets.
- `python -m unittest discover -s tools -p test_check_vocabulary.py`: six tests.
- `python tools/check_conversations.py`: 1,000 conversations and a current index.
- `cargo fmt --check`, `cargo check --locked`, `cargo test --locked`:
  all 92 Rust tests pass, including score migration and homograph preservation.
- `cargo clippy --locked --all-targets --all-features`: passes.
- `build_and_sync.bat`: WebAssembly build and generated GitHub Pages output pass.
