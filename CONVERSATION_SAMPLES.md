# Conversation samples for review

These are three content drafts, each with six exchanges (12 messages). They are
not connected to the running app yet. Review the Croatian, translations, tone,
length, and repetition before producing a larger library.

Agreed interaction: show the partner's message first. Tapping reveals the learner's
masked reply and starts recall timing. Reveal/rate works as before; tapping a
message toggles English translations for the whole conversation. The precise tap
targets for those two actions will need to be distinct in the eventual interface.

The text below is deliberately unmasked for content review. Review targets are
chosen at runtime from the learner's vocabulary and scores, not fixed in these
drafts. Partner words do not contribute to selection weights or coverage.

## 1. Meeting for coffee

Draft ID: `0070`. Two friends meet at a café and decide what to order.

| Exchange | Partner — Croatian | Learner — Croatian | Partner — English | Learner — English |
| --- | --- | --- | --- | --- |
| 1 | Bok! Jesi li već naručio? | Još nisam. Čekam te. | Hi! Have you ordered yet? | Not yet. I'm waiting for you. |
| 2 | Što ćeš popiti? | Ja bih jednu kavu, molim. | What will you have to drink? | I'd like a coffee, please. |
| 3 | Sa šećerom ili bez? | Obično pijem kavu bez šećera. | With sugar or without? | I usually drink coffee without sugar. |
| 4 | Hoćeš li i nešto za jelo? | Da, jedan sendvič, molim. | Would you like something to eat as well? | Yes, a sandwich, please. |
| 5 | Idemo poslije malo prošetati? | Može, ali prvo želim popiti kavu. | Shall we go for a short walk afterwards? | Sure, but first I want to drink my coffee. |
| 6 | Dogovoreno. Danas ja plaćam. | Hvala! Sljedeći put plaćam ja. | Agreed. I'm paying today. | Thank you! Next time I'm paying. |

The exact learner token `kavu` occurs **three times** (exchanges 2, 3, and 5).
The partner's appearances of any vocabulary are excluded from those counts.

## 2. Buying bread and milk

Draft ID: `0071`. A customer buys a few things at a small shop.

| Exchange | Partner — Croatian | Learner — Croatian | Partner — English | Learner — English |
| --- | --- | --- | --- | --- |
| 1 | Dobar dan! Što trebate? | Dobar dan! Trebam kruh i mlijeko. | Good afternoon! What do you need? | Good afternoon! I need bread and milk. |
| 2 | Bijeli ili crni kruh? | Imate li još crnog kruha? | White or brown bread? | Do you have any brown bread left? |
| 3 | Imamo, upravo je stigao. | Odlično, uzet ću jedan kruh. | We do; it has just arrived. | Great, I'll take one loaf. |
| 4 | Koliko mlijeka želite? | Dvije boce, molim. | How much milk would you like? | Two bottles, please. |
| 5 | Je li to sve? | Još trebam kavu za sutra. | Is that everything? | I also need coffee for tomorrow. |
| 6 | To je ukupno osam eura. | Izvolite. Hvala i doviđenja. | That's eight euros altogether. | Here you are. Thank you and goodbye. |

The exact learner token `kavu` occurs **once** (exchange 5). `kruh` occurs twice,
while the distinct entry `kruha` occurs once. Those bread forms must not be merged.

For a target of `kavu`, when both `0070` and `0071` have the lowest equal
completed-read count, their weights are **3:1**, giving 75% and 25% selection
probabilities within this two-conversation sample. If either has been completed
more often, it is blocked until the other catches up; the frequency weighting is
only applied after that filter.

## 3. Plans for tomorrow

Draft ID: `0072`. Two friends arrange a walk after work.

| Exchange | Partner — Croatian | Learner — Croatian | Partner — English | Learner — English |
| --- | --- | --- | --- | --- |
| 1 | Imaš li planove za sutra? | Još nemam. Što predlažeš? | Do you have plans for tomorrow? | Not yet. What do you suggest? |
| 2 | Hoćemo li prošetati uz more? | Rado, ali sutra moram raditi. | Shall we go for a walk by the sea? | I'd like to, but I have to work tomorrow. |
| 3 | Kada završavaš s poslom? | Radim do tri, a poslije imam vremena. | When do you finish work? | I work until three, and afterwards I have time. |
| 4 | Onda možemo krenuti oko četiri. | Može. Mislim da će sutra biti lijepo vrijeme. | Then we can head out around four. | Sure. I think the weather will be nice tomorrow. |
| 5 | Hoćeš li ponijeti nešto za jelo? | Da, mogu ponijeti kruh i sir. | Will you bring something to eat? | Yes, I can bring bread and cheese. |
| 6 | Odlično! Vidimo se ispred tvoje zgrade. | Dogovoreno, vidimo se sutra u četiri. | Great! See you outside your building. | Agreed, see you tomorrow at four. |

The exact learner token `sutra` occurs **three times** (exchanges 2, 4, and 6).
It occurs once in the learner replies of `0071`. Its occurrence in this
conversation's first partner message is not counted.

## Content decisions to review

- Is this conversational style and six-exchange length appropriate?
- Is the repeated vocabulary useful and natural enough? The repetition is
  intentional, but frequency weighting should not encourage awkward filler.
- These are everyday situations, not a fixed beginner level. Grammar, other
  unseen words, and partner messages can still be difficult even when the target
  is simple. The final library needs a difficulty/known-vocabulary policy.
- Exact spelling/form matching is retained. Sentence-initial capitalization is
  checked separately below rather than silently changing the tokenizer.
- Whole-message English translations are contextual. They should be reviewed
  independently of the dictionary's short per-word hints, which can be ambiguous.
- Three samples do not establish full vocabulary coverage. Before mass generation,
  coverage must be checked across all learner replies against every vocabulary
  entry, with missing forms reported rather than assumed to be present.

## Exact vocabulary check

Checked against the current `assets/croatian.json`, using the current tokenizer's
case-sensitive separators. Counts include learner replies only; they do not merge
grammatical forms or silently lowercase words. All learner tokens either match an
existing entry exactly or differ only by sentence-initial capitalization. No new
vocabulary entries were added.

| Conversation | Learner token occurrences | Exact matches now | Capitalized forms with lowercase entries |
| --- | ---: | ---: | --- |
| Meeting for coffee | 29 | 21 | `Da`, `Hvala`, `Ja`, `Još`, `Može`, `Obično`, `Sljedeći`, `Čekam` |
| Buying bread and milk | 28 | 20 | `Dobar`, `Dvije`, `Hvala`, `Imate`, `Izvolite`, `Još`, `Odlično`, `Trebam` |
| Plans for tomorrow | 36 | 28 | `Da`, `Dogovoreno`, `Još`, `Mislim`, `Može`, `Radim`, `Rado`, `Što` |

These capitalized forms would currently remain visible unless their exact spelling
is an entry. For example, `Jo?` does not match `jo?`. Before mass generation, decide
whether sentence-initial capitalization should map to the existing lowercase
entry while still keeping grammatical forms such as `kruh` and `kruha` separate.
No tokenizer or scoring change is included in these drafts.
