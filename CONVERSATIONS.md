# Conversation library

**Partial draft, not full vocabulary coverage.** The library contains more than
100 conversations, with five to eight exchanges each. The current counts and
every uncovered form are recorded in `CONVERSATION_COVERAGE.json`; that report
and the strict check are authoritative after edits. Reaching 100 conversations
does not complete the task: every corrected vocabulary form must appear in a
learner reply.

[CONVERSATION_EXPANSION_1000_REPORT.md](CONVERSATION_EXPANSION_1000_REPORT.md) records the
latest expansion: 100 new conversations, each adding at least two uncovered forms.
The library now has **1,000 conversations**, covering **5,673 of 8,574 forms (66.2%)**.
This batch adds **246 previously uncovered forms**. Existing dialogue and IDs were
preserved. No conversation contributes
fewer than two first appearances in ascending ID order. This is an authoring metric,
not the runtime review order or a requirement that each form appear in only one
conversation.

[CONVERSATION_EXPANSION_900_REPORT.md](CONVERSATION_EXPANSION_900_REPORT.md) preserves
the preceding 800-to-900 batch and its 261-form coverage gain.

[CONVERSATION_EXPANSION_800_REPORT.md](CONVERSATION_EXPANSION_800_REPORT.md) preserves
the preceding 700-to-800 batch and its 266-form coverage gain.

[CONVERSATION_EXPANSION_REPORT.md](CONVERSATION_EXPANSION_REPORT.md) records the
preceding 47-conversation expansion and requested ID compaction back to 700 entries.

[CONVERSATION_REVIEW_REPORT.md](CONVERSATION_REVIEW_REPORT.md) preserves the preceding
cleanup audit, which removed 47 zero-contribution conversations and expanded all
59 one-form conversations. Its tables use the IDs from before the later renumbering.

[CONVERSATION_BATCH_REPORT.md](CONVERSATION_BATCH_REPORT.md) preserves the historical
audit of the preceding 100-conversation expansion, before pruning. The earlier
repetitive 400-conversation draft was not published. The 1,000-conversation milestone
is now reached. Full vocabulary coverage remains outstanding: 2,901 forms are missing.

The drafts cover food, shopping, directions, travel, home, social plans, work,
study, health, leisure, family, and past experiences. The first authoring batch used
four situational/lexical variants of 25 authored scenarios, plus the three approved
samples. Some of those variants were removed in the coverage review. Later
batches add independently written stories about misunderstandings, memories,
relationships, school, travel, community life, and work. Hotel arrivals include
separate early-arrival, late-arrival, noise, and facilities scenarios.

This content is not connected to the current review UI. It does not change scoring,
or sentence selection. The vocabulary corrections below do migrate the affected
saved score keys. Browser-model code removed earlier has not been restored.
The conversations are authored directly in this session, without other models or
external APIs. Earlier local-model probes contributed no library content, and the
unused API-authoring helper has been removed. Native-speaker review remains
appropriate before publishing the drafts as teaching material; automated checks
do not certify language quality.

## Runtime JSON

Source file: `assets/conversations.json`.

```json
{
  "schema_version": 1,
  "source_language": "hr",
  "translation_language": "en",
  "content_status": "draft",
  "conversations": [
    {
      "id": "0070",
      "title": "Meeting for coffee",
      "topic": "food_and_drink",
      "exchanges": [
        {
          "partner": {"hr": "Što ćeš popiti?", "en": "What will you have to drink?"},
          "learner": {"hr": "Ja bih jednu kavu, molim.", "en": "I'd like a coffee, please."}
        }
      ]
    }
  ],
  "word_index": {
    "kavu": [{"c_id": "0070", "hits": 3}]
  },
  "coverage": {"coverage_complete": false}
}
```

The snippet abbreviates the conversation and coverage fields. The actual file has
5-8 full exchanges per conversation and a computed index/coverage summary.

IDs are numeric strings, padded to at least four digits: `0001`, `0002`, and so on,
continuing with `10000` after `9999`. At the user's request, the surviving 653
conversations were renumbered consecutively and the new 47 appended. A subsequent
series of 100-conversation batches preserves those IDs and adds `0701`–`1000`. The current
IDs are **`0001`–`1000`**, matching their positions; the next new ID is **`1001`**.
The earlier policy of leaving retired gaps was superseded by this compaction.

Renumbering requires rebuilding every `c_id` in the word index and updating document
references. The three approved samples now have IDs `0070`, `0071` and `0072`.
Historical audit tables retain their old snapshot IDs and should be matched by title.
Conversation progress is not integrated yet; any later renumbering after ID-based
read counts exist would also require migrating those stored references.

Each exchange presents the partner
first, followed by the learner after a tap. Only then does recall timing start.
Both messages have independent Croatian text and contextual English translations,
supporting the global translation toggle without regenerating content.

## Index and matching

`word_index[word]` lists conversations containing that exact vocabulary form in
learner replies. Each compact entry has `c_id` (the conversation's `id`) and
`hits` (the actual learner occurrence count). Keep these short keys when
regenerating the index. Partner messages never
contribute. For example, the approved `0070` has three learner occurrences of
`kavu`; `0071` has one. This supports 3:1 weighting after filtering to the
lowest completed-read count. User read counts belong in progress storage, not in
this shared content file.

Lookup lowercases Unicode text without changing display text. It does not remove
diacritics or merge grammatical forms: `Kruh` maps to `kruh`, but `kruha` remains a
separate entry, and `čaj` does not become `caj`. The index uses the existing Rust
token separators. The current review UI's tokenizer has not been changed to use
this new library/matching policy yet.

`CONVERSATION_COVERAGE.json` records every missing form and unlisted learner token.
Nothing is counted just because it appears in a partner message, English
translation, title, or index metadata. Do not count substrings as coverage or
manufacture word-list conversations to cover difficult forms.

## Reviewed vocabulary corrections

The user approved correcting misspellings and merging them when the corrected
form already exists. The three initially flagged entries were merged:

| Removed entry | Existing corrected entry |
| --- | --- |
| `covek` | `čovjek` |
| `pomoči` | `pomoći` |
| `o.k.` | `okej` |

`assets/vocabulary_corrections.json` records these edits and their affected sentence
indexes. Vocabulary references are combined in sorted order; sentence text and
explanations use the corrected forms. There are now 8,574 vocabulary entries.
Other grammatical, regional, and accented forms have not been merged merely for
being similar. Additional questionable entries still require contextual review.

Progress readers and writers map those three former spellings to the corrected
key. When both have scores, the most recently reviewed record wins; equal review
times retain the higher score. Scores are not added together. Reading does not
write storage; failed saves preserve the existing stored snapshot, and legacy
progress keys remain intact. The snapshot schema and normal scoring are unchanged.

## Checking and editing

After editing/adding JSON conversations:

```powershell
python tools/check_conversations.py --write-index
python tools/check_conversations.py
python tools/check_conversations.py --require-full-coverage
python -m unittest discover -s src/tests -p test_conversations.py
```

The strict command currently exits **1**, intentionally, because coverage is
incomplete. It must exit 0 before claiming that every vocabulary entry is covered.
Ordinary validation checks IDs, exact duplicate conversations, 5-8 exchanges,
nonempty bilingual messages, and index/summary consistency. Passing those checks
does not imply complete coverage or linguistic correctness.

Keep `content_status` as `draft` while language/coverage review is pending. Existing
sentences remain the only runtime content until conversation support is implemented.

## Continuing the writing

Natural, correct Croatian and faithful contextual translations take precedence
over vocabulary coverage. Use related uncovered forms only when they fit the
situation; rewrite or discard clumsy sentences rather than forcing a cluster.
For the current curated library, count first appearances in ascending numeric ID
order. Remove conversations contributing none, provided all existing coverage is
preserved. Review one-form conversations for natural additions from the uncovered
list, and update their translations and following replies as needed. Do not force
awkward sentences merely to meet a numerical quota. Recount the entire library
after edits because earlier additions can change later contribution counts.
Aim for at least two previously uncovered forms in each new conversation unless
doing so would make the language unnatural; document any exception rather than
adding filler.

Read the remaining forms in `CONVERSATION_COVERAGE.json`, choose a coherent
situation that can naturally include some of them, and write both speakers and
their translations. Keep the exchange understandable without surrounding stories.
Do not use another model or an external API for authoring. Append new conversations
and rerun the checker to recompute actual coverage. Only renumber when explicitly
requested, updating all references together.

The index is derived content, not an authoring target to edit manually. All
5–8 exchanges must be complete even when the target word appears only once.
Six Python unit tests cover matching, occurrence counts, rejected structures,
real message coverage, and numeric IDs (including gaps and values above `9999`).
