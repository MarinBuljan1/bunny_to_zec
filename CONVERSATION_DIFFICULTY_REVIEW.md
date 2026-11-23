# Conversation difficulty and selection

Started 18 November 2026. The editorial assignments now drive runtime status
gates. Historical review and expansion snapshots below record how the content
and rules were chosen; the following section describes current selection.

## Runtime status gates: 23 November 2026

| Displayed status | Learned-word-equivalent score | Allowed bands | Eligible conversations |
| --- | --- | --- | ---: |
| Beginner | Below 100 | 1 | 100 |
| Novice | 100 to below 300 | 1–2 | 461 |
| Conversational | 300 to below 1,000 | 1–3 | 1,005 |
| Intermediate and above | 1,000+ | 1–4 | 1,126 |

The display and selector share thresholds. This is the existing capped-score sum
divided by 864,000, not a count of previously encountered forms. Gates are based
on current progress rather than a permanently saved highest level.

1. Keep review-priority targets only if they occur in an unlocked learner reply.
   Avoid words from the previous reply when another eligible target exists.
2. Among unlocked matches for that target, prefer the lowest completed-read count.
3. Break ties by lowest band, lowest average vocabulary rank, then ID. Occurrence
   counts remain validated, but selection is deterministic rather than weighted.

Harder-only targets wait; the selector repeats eligible material rather than
opening a locked band. Scores and review formulas are unchanged. A running
conversation can finish after a score decrease. On reload, a saved cursor in a
locked band is cleared with a notice, preserving scores, read counts and English
preference. A new conversation then follows the current level.

Conversation schema 2 embeds a required `band` on each runtime conversation, so
offline selection needs no separate request. The review JSON retains the full
editorial assignments and coverage requirements; the checker verifies embedded
bands against it. Invalid bands and libraries without reviewable Starter content
are rejected. There is no progress-schema migration or dialogue/ID change.

The rubric remains an editorial judgement, not a certified proficiency rating.
Average vocabulary rank orders conversations within equal bands, rather than
claiming to measure all aspects of Croatian difficulty.

## Conversational coverage expansion: 23 November 2026

Added **41 conversations** (`1086`–`1126`): **21 Everyday** and **20 Connected**,
with six to seven exchanges each. Existing conversations and IDs `0001`–`1085`
are unchanged. Every new message has Croatian text and an English translation.

**Bands 1–3 together now cover all first 1,500 vocabulary forms in learner replies.**
Before this expansion they covered 1,341; 29 missing forms appeared only in Band 4,
and 130 were absent from learner replies throughout the library. All 159 gaps are
now covered by new learner replies in Bands 2 or 3, without reclassifying existing
Band 4 conversations. The earlier guarantees also remain: Band 1 covers all first
200 forms, and Band 2 independently covers all first 500.

| Band | Description | Conversations | Cumulative conversations | Cumulative vocabulary forms in learner replies |
| --- | --- | ---: | ---: | ---: |
| 1 | Starter | 100 | 100 | 704 |
| 2 | Everyday | 361 | 461 | 3,078 |
| 3 | Connected | 544 | 1,005 | 5,411 |
| 4 | Complex | 121 | 1,126 | 5,834 |

This batch adds **149 globally uncovered forms**: the 130 missing early forms
and 19 additional forms beyond the first 1,500. Global coverage is now
**5,834/8,380 (69.6%)**, with 2,546 forms still uncovered. As with the beginner
expansion, the main goal is availability in the appropriate bands rather than
maximising global first appearances in every conversation.

Situations include family introductions, errands, job interviews, travel mishaps,
home repairs, hobbies and community events. More demanding connected stories
include a borrowed phone, a detective novel, family history and a changed dinner
booking. Informal language and swearing have corresponding English translations;
regional message wording is discussed in context rather than silently presented
as the preferred wording for a Croatian hotel enquiry.

`tools/check_conversation_bands.py` now checks both independent and cumulative
coverage goals. The Band 3 requirement explicitly includes earlier bands; Band 2's
500-form requirement remains independent. Tests verify that partner messages,
different grammatical forms and later bands cannot fill an eligible coverage gap.
The review JSON records current assignments, counts, requirements and source hashes.
At the time of this content expansion, runtime status gates were still pending;
they are now implemented as described above.

## Beginner coverage expansion: 23 November 2026

The following counts describe the earlier 1,085-conversation snapshot.

Added **85 independently authored conversations** with five to six exchanges each:
67 Starter conversations (`1001`–`1067`) and 18 Everyday conversations
(`1068`–`1085`). The original 1,000 conversations and their IDs are unchanged.
New topics include introductions, classroom routines, transport, simple purchases,
food, home life and family visits. Both speakers have Croatian text and English
translations.

| Band | Description | Conversations | Cumulative conversations | Cumulative vocabulary forms in learner replies |
| --- | --- | ---: | ---: | ---: |
| 1 | Starter | 100 | 100 | 704 |
| 2 | Everyday | 340 | 440 | 2,962 |
| 3 | Connected | 524 | 964 | 5,226 |
| 4 | Complex | 121 | 1,085 | 5,685 |

**Band 1 covers all first 200 vocabulary forms**, including all first 100.
**Band 2 independently covers all first 500 forms**, including all first 300;
this guarantee does not rely on Band 1. Coverage means an exact lowercase match
in a **learner reply** using the app's tokenizer and vocabulary file order.
Grammatical forms remain separate. Punctuation matters: a word immediately
followed by a colon does not currently count as that exact form, so the new
coverage sentences use supported separators.

The library now covers 5,685 of 8,380 forms overall, 12 more than before. The
purpose of this expansion is early-band availability and variety: most early
forms were already present elsewhere in the original library. Unlike the older
batches aimed at filling global coverage gaps, these conversations are not
required to introduce two globally unseen forms each.

Assignments, updated counts, source hashes and coverage requirements are in
[CONVERSATION_DIFFICULTY_BANDS.json](CONVERSATION_DIFFICULTY_BANDS.json).
Run these checks after content or assignment edits:

```powershell
python -X utf8 tools/check_conversations.py
python -X utf8 tools/check_conversation_bands.py
```

The second check detects unassigned IDs, stale hashes/summaries, missing early
forms and a Starter pool below 100 conversations. Automated checks establish
coverage and structural consistency; the difficulty and language review remains
editorial. Status gates and target selection within allowed bands are still a
separate implementation task. These pools provide content for the proposed
Beginner/Novice/Conversational unlocks, without changing today's runtime selector.

## Original 1,000-conversation grouping: 23 November 2026

The following counts and starter inventory describe the snapshot **before** the
85-conversation expansion above.

All **1,000 original conversations** received a provisional editorial band in
[CONVERSATION_DIFFICULTY_BANDS.json](CONVERSATION_DIFFICULTY_BANDS.json).
That file is a review artifact; the app does not load it. The initial grouping
did not change conversation content, review logic, saved progress or deployment output.

| Band | Description | Conversations | Share |
| --- | --- | ---: | ---: |
| 1 | Starter | 33 | 3.3% |
| 2 | Everyday | 322 | 32.2% |
| 3 | Connected | 524 | 52.4% |
| 4 | Complex | 121 | 12.1% |
| **Total** | | **1,000** | **100%** |

With the proposed status gates, the available pools would be:

| App status | Learned-word-equivalent score | Allowed bands | Conversations available | Distinct vocabulary forms in learner replies |
| --- | --- | --- | ---: | ---: |
| Beginner | Below 100 | 1 | 33 | 355 |
| Novice | 100 to below 300 | 1–2 | 355 | 2,835 |
| Conversational | 300 to below 1,000 | 1–3 | 879 | 5,206 |
| Intermediate and above | 1,000+ | 1–4 | 1,000 | 5,673 |

The vocabulary column is exact-form coverage, not a prediction of how quickly
someone will learn those forms or reach a status. A learned-word-equivalent score
is not simply the number of forms encountered.

### How the grouping was reviewed

The complete English exchanges were read for the task, meaning, narrative links,
inference and specialist concepts. Croatian message-length and vocabulary
measurements were available across the library. Every proposed Band 1 conversation
and selected boundary cases were re-read in full Croatian to check the actual
wording and grammatical demands. Seven provisional starter candidates were moved
to Band 2 after that check: `0337`, `0373`, `0514`, `0912`, `0962`, `0972`, `0974`.

This is an editorial classification using the rubric below, not a validated
Croatian proficiency assessment. All entries are assigned, but boundaries between
bands remain judgement calls. The 17 pilot examples retain their proposed bands.

Band 1 was interpreted conservatively: predictable exchanges, short usable
phrases, concrete meanings and limited need to follow implied motives or linked
explanations. Formulaic future forms, polite requests and imperatives do not
automatically disqualify a starter conversation. Nor does a single less common
but concrete word. A familiar title does not automatically qualify one either.

Examples of the distinction:

- `0005`, a predictable restaurant order, is Band 1. `0006` shares that setting
  but adds a longer substitution request and explanation, so is Band 2.
- `0074`, meeting a new colleague, is Band 2 despite its introductory setting:
  the dialogue also discusses a manager's absence, return and later plans.
- `0264`, finding the wrong meeting floor, is Band 2: simple everyday subject,
  but a past-event misunderstanding rather than a predictable transaction.
- `0073`, clearing up a missed call, is Band 3 because the speakers reconstruct
  events, reported information and misunderstood intentions.
- `0819`, a confidential work story, remains Band 4: short messages express an
  abstract distinction between personal trust and confidentiality.

### Starter inventory and implications

The 33 Band 1 IDs are:

`0001`, `0002`, `0003`, `0004`, `0005`, `0007`, `0009`, `0011`, `0013`,
`0014`, `0015`, `0016`, `0018`, `0020`, `0021`, `0025`, `0026`, `0027`,
`0028`, `0029`, `0030`, `0032`, `0036`, `0050`, `0055`, `0070`, `0071`,
`0325`, `0349`, `0515`, `0574`, `0828`, `0895`.

Several early entries are variations of the same scenario, so 33 conversations
provide less variety than 33 unrelated situations would. The current library was
expanded to cover vocabulary, not to supply a large introductory course. More
starter dialogue would be worthwhile before enforcing a long beginner-only phase.

The starter pool covers **70 of the first 100 vocabulary forms** in list order,
despite covering 355 forms overall. Examples absent from starter learner replies
include `znam`, `si`, `on`, `zašto`, `rekao`, and `možda`. The unlock rule therefore
needs to apply while selecting the review target as well as the conversation;
otherwise an unavailable early target could leave the selector with no eligible
conversation. These words need not be deleted or changed: they can be deferred
until an allowed conversation contains them, or covered in additional starter
dialogue. This choice is still for implementation later.

Band 1's 355 forms exceed the 100-point unlock threshold in principle. That is
not a guarantee of a smooth learning path: it depends on repetitions, score
growth, recall and which forms are introduced. Similarly, the other cumulative
pools contain more forms than their next unlock thresholds.

### Validation

Checked that each of the 1,000 existing IDs appears exactly once, every rating is
an integer from 1 to 4, counts sum to 1,000, and cumulative pools/coverage match
the actual learner-only word index. The JSON records the source file hash so a
later dialogue edit can be detected before reusing these assignments. This
documentation/data-only review does not require rebuilding the app.

## Findings

The current selector first chooses a review-priority word. It then keeps only
conversations containing that word in a learner reply and having the lowest read
count among those matches. Finally, it samples randomly with weight:

`learner occurrences of the target / average vocabulary rank`

The average includes both speakers and every word occurrence. Words outside the
vocabulary receive its highest rank. Lower scores make selection more likely;
they never exclude harder conversations.

This explains why an advanced topic can appear early. Another problem is that
common words can express complicated ideas. An average rank cannot reliably
measure sentence structure, inference, emotional nuance or topic complexity.
Exact grammatical forms also have separate vocabulary ranks, so a perfectly
ordinary inflection can make an otherwise simple conversation look difficult.

For a completely new learner, the first target is normally `je`, which occurs in
learner replies in 772 conversations. `Da` occurs in 691. These common words give
the selector a broad range of topics; their presence does not indicate that a
conversation suits a beginner. Existing progress or the previous review words
can change which target comes first.

## Suggested ranking rubric

Use an editorial difficulty band for the complete Croatian dialogue, followed
by a relative order within the band. Read both speakers, not just the learner
replies or the English title.

| Band | Description | What to examine |
| --- | --- | --- |
| 1: Starter | Concrete, predictable situations with short, reusable replies | Ordering, quantities, directions; limited inference and few clauses per message |
| 2: Everyday | Familiar situations requiring somewhat more independent language | Personal updates, descriptions, straightforward reasons and preferences |
| 3: Connected | Narratives, misunderstandings and explanations involving several ideas | Reported speech, past/future connections, conditions, implied reasons |
| 4: Complex | Abstract reasoning, nuanced interaction or a substantial specialist vocabulary load | Confidentiality, evidence, ethical arguments, technical explanations, elaborate descriptions |

These bands are relative to this library, **not certified CEFR levels**. Even the
starter examples contain future forms or imperatives. A familiar topic is not
enough: long or grammatically demanding shopping dialogue could rank higher.
Likewise, a conversation about science can be simple if the actual language is
simple. The strongest obstacle should matter; an easy opening must not hide a
difficult reply later in the conversation.

Review in this order:

1. Communicative task and how predictable the next reply is.
2. Sentence structure and grammatical demands throughout the conversation.
3. Vocabulary burden, specialist words and the hardest individual messages.

Use automatic measurements to flag candidates for review, rather than treating
the measurements as an authoritative language level. Later, learner familiarity
could break ties between similarly ranked conversations; that is optional and
does not need to be part of the initial implementation.

## Pilot ranking

All messages in the following 17 conversations were read for this pilot. The
order is a provisional editorial judgement, not an automatically calculated
score. It should be reviewed before annotating the whole library.

The numerical columns reproduce the current vocabulary-rank calculation and
count Croatian words per individual message across both speakers. Lower mean
vocabulary rank means more common forms in this app's list. These measurements
cover the current 1,000-conversation library; the manual ratings cover only the
17 rows below.

| Pilot order | ID | Conversation | Band | Mean vocab rank | Mean / longest message, words | Reason |
| ---: | --- | --- | ---: | ---: | --- | --- |
| 1 | 0001 | Ordering at a café: coffee and a sandwich | 1 | 775 | 5.2 / 6 | Predictable requests, short replies, concrete situation |
| 2 | 0007 | At the market: apples and pears | 1 | 1,460 | 4.7 / 8 | Routine quantities and shopping, with some inflected food words |
| 3 | 0015 | Asking for directions: the station | 1 | 1,934 | 4.5 / 7 | Short, concrete directions; commands and future forms add some demand |
| 4 | 0082 | Meeting an old friend again | 2 | 400 | 7.8 / 13 | Familiar subject, but longer replies connect past events and future plans |
| 5 | 0321 | Choosing wood for a small shelf | 2 | 3,100 | 6.0 / 9 | Concrete task with short replies; materials, measurements and preferences add vocabulary |
| 6 | 0073 | Clearing up a missed call | 3 | 659 | 8.1 / 11 | Past events, reported information and a misunderstood intention |
| 7 | 0131 | Two sisters disagree about family plans | 3 | 764 | 8.2 / 12 | Explaining obligations, correcting assumptions and conditional plans |
| 8 | 0993 | An interview starts with a remembered family scene | 3 | 1,469 | 6.8 / 10 | Embedded questions, remembered events and uncertainty about details |
| 9 | 0161 | Doubts about moving abroad | 3 | 1,102 | 9.0 / 14 | Reasons, worries and hypothetical alternatives across several clauses |
| 10 | 0114 | Setting a boundary with a friend | 3 | 816 | 9.0 / 14 | Explaining a boundary requires tracking reasons, ownership and emotional nuance |
| 11 | 0128 | Checking a rumour before repeating it | 3 | 819 | 9.1 / 14 | Reported speech, uncertain sources and reasoning about incomplete information |
| 12 | 0819 | A friend respects a work story that cannot be shared | 4 | 1,167 | 6.8 / 9 | Short sentences still express abstract confidentiality and limits of trust |
| 13 | 0921 | A crime novel gives two witnesses incompatible accounts | 4 | 1,521 | 7.0 / 11 | Evidence, unreliable memory and checking an assumption |
| 14 | 0989 | An online disagreement stops being about the topic | 4 | 1,972 | 7.4 / 13 | Abstract disagreement, reporting rules and evaluating one's own contribution |
| 15 | 0601 | An astronomy class waits for a clear sky | 4 | 2,544 | 7.2 / 11 | Scientific terminology and an explanation of astronomical distances |
| 16 | 0191 | A science project that failed honestly | 4 | 1,733 | 9.6 / 16 | Experimental comparisons, research ethics, limitations and hypothetical improvements |
| 17 | 0201 | A cover for a fantasy novel | 4 | 2,117 | 10.6 / 18 | Dense descriptions, subordinate clauses, intentions and hypothetical consequences |

The clearest warning sign is 0082 versus 0001: the existing average ranks the
reunion easier than the café order, despite longer and more involved replies.
Likewise, 0819 uses relatively common words and short messages, but its abstract
reasoning is not a good opening exercise. Conversely, the shelf conversation's
less common concrete vocabulary does not by itself make it the hardest dialogue
in this sample.

## Selection options to decide after ranking

**Smallest change: deterministic easiest eligible conversation.** Keep the target
word and minimum-read-count rules. Replace weighted sampling with selection of
the lowest difficulty band, then lowest editorial order, then ID for a stable tie.
Target-occurrence counts would no longer determine selection probability. They
can remain in the word index for its current validation and occurrence information.

This improves the first selection among equal-read alternatives. It does **not**
guarantee gradual difficulty: once an easy conversation has been read, an unread
hard conversation containing the next target outranks it under the read-count
rule. The same applies when a target only occurs in hard conversations.

**Stronger beginner progression: difficulty eligibility before read counts.**
Keep the review-priority target when there is an appropriate-level match, first
filter to allowed difficulty bands, then apply minimum read count and choose the
easiest. This intentionally relaxes the old requirement that every conversation
containing a word be read equally often before any repeats. Advanced conversations
remain available later rather than being forced in early for the sake of fairness.

Before implementing this option, decide what happens when the next target has no
appropriate-level conversation: defer that target while another suitable review
is available, or permit a harder fallback. A strict beginner introduction would
defer it. If none of the priority words have suitable conversations, the app must
repeat suitable material, widen the allowed band, or obtain more beginner content;
ranking alone cannot create missing easy dialogue.

Do not set arbitrary score thresholds for unlocking bands yet. First rank a
larger sample, inspect coverage of common targets in the starter/everyday pool,
and agree on whether repeats or harder new material should win. Progression could
eventually follow performance or be an explicit user-selected level.

## Recommended next step

Use the pilot to agree on the rubric, then annotate the rest of the library.
Keep IDs and dialogue unchanged, and store difficulty metadata separately from
vocabulary priority. Compare the deterministic selector's proposed early choices
with the current selector before connecting the metadata to the app. No runtime
implementation is included in this review.
