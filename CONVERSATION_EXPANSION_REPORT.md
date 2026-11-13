# Conversation expansion: restoring the total to 700

This is the historical 47-conversation batch. The next expansion to 800 is recorded
in [CONVERSATION_EXPANSION_800_REPORT.md](CONVERSATION_EXPANSION_800_REPORT.md).

This batch adds **47 directly authored conversations**, each with five complete
Croatian/English exchanges. No other model or external API was used.
Natural Croatian and faithful translations take precedence over coverage density.
Every new conversation contributes **at least two previously uncovered forms**;
no exception was needed for this batch.

- Before: **653 conversations; 4,750/8,574 forms covered**.
- Added: **47 conversations; 150 distinct previously uncovered forms**.
- Average gain: **3.19 new forms per added conversation**.
- After: **700 conversations; 4,900/8,574 forms covered (57.1%)**.
- Remaining: **3,674 forms**. Full vocabulary coverage is incomplete.
- Existing 653 conversations: **dialogue, translations, titles, topics and order unchanged**.
- All 700 conversations still contribute at least two first appearances in ID order.

Coverage counts exact lowercased vocabulary forms in learner replies only.
Accents and grammatical forms stay separate. A repeated form is credited only to
the first new conversation introducing it; partner messages, English translations
and metadata do not count.

## Consecutive IDs

At the user's explicit request, the existing 653 conversations were renumbered in
their existing order as **`0001`–`0653`**. The new conversations use **`0654`–`0700`**.
The 654th conversation therefore has ID `0654`. There are no gaps or duplicate IDs.
The next new ID is **`0701`**.

The selection index was fully rebuilt using `c_id` / `hits`. Sample references
were updated: the café, shop and planning samples are now `0070`, `0071` and `0072`
(previously `0101`, `0102` and `0103`). Their dialogue and occurrence weights are
unchanged. Conversations are not yet integrated into the review UI, so this change
does not rewrite or migrate app progress.

The earlier review and batch reports retain the IDs from their historical
snapshots. Those IDs are not current references; use conversation titles to match
their entries to the current library. The former gap/retired-ID policy was
superseded by this requested compaction.

## Validation

The content validator checks unique IDs, duplicate complete dialogues, 5–8
exchanges, bilingual messages and the computed index. A separate recount checked
every `c_id` / `hits` entry, contribution counts, preservation of old coverage and
the unchanged content of the original 653 records. The writer reviewed the new
dialogues and translations; automated checks do not certify language quality.
The library remains a draft pending independent language review.

## Added conversations

| Current ID | Situation | New forms | Forms first covered |
| --- | --- | ---: | --- |
| 0654 | A farm visit has to fit feeding time | 6 | daju, farma, farmu, funkcionira, nahraniti, čizme |
| 0655 | A reader finds a familiar name in an old magazine | 5 | krije, mladić, naišao, novine, časopis |
| 0656 | Two friends choose a language course | 3 | ispočetka, japanski, španjolski |
| 0657 | A museum blacksmith lets visitors try a tool | 3 | majstor, snaga, čekić |
| 0658 | A guest discovers cat hair on a dark coat | 3 | dlake, mačku, čuvao |
| 0659 | A retired sailor recognises a model ship | 4 | brodove, brodovi, razdoblje, strpljenja |
| 0660 | A dream keeps returning in different places | 3 | sanjam, snove, snovi |
| 0661 | A neighbourhood story reaches the evening news | 3 | mediji, novinari, vijestima |
| 0662 | A flower brooch needs a different place | 2 | cvijet, haljini |
| 0663 | A grandfather waits to meet a new baby | 4 | beba, rodila, vrti, željeli |
| 0664 | A home recording captures the building works | 2 | buka, glasove |
| 0665 | A rainbow appears after the picnic is packed | 5 | brda, duga, ljepše, munja, uhvatiti |
| 0666 | An actor decides whether to audition | 3 | audiciju, glumio, iskreni |
| 0667 | A flatmate loses the television remote | 4 | baterija, daljinski, novčića, samom |
| 0668 | A neighbour borrows a drill for one small shelf | 2 | garažu, prašine |
| 0669 | A letter mentions an unfamiliar family profession | 3 | dolini, inženjer, terenu |
| 0670 | A family avoids buying the same present twice | 2 | godišnjicu, slave |
| 0671 | A child counts the birds without counting one twice | 3 | broji, lijeve, odande |
| 0672 | A reading group disagrees about a rude character | 4 | budale, divim, grub, posljedice |
| 0673 | A friend admits being jealous of good news | 2 | ljubomoran, uspjeha |
| 0674 | A group chooses a flatter route for a walk | 3 | družiti, postavljati, toalet |
| 0675 | An amateur cook finally buys a proper apron | 6 | majice, rukom, tragova, uništio, zaštitim, zidovima |
| 0676 | A commuter practises driving a different car | 3 | mojem, ulicama, vozilu |
| 0677 | A shopkeeper starts recognising a new regular | 2 | dućan, sapun |
| 0678 | A trial subscription is not as free as it looked | 4 | besplatno, ispalo, otkazati, računa |
| 0679 | A child asks a magician to repeat a trick | 2 | trik, trikove |
| 0680 | A frozen lake looks safer than it feels | 3 | led, ledu, sreću |
| 0681 | A child mistakes a ruin for a pile of stones | 4 | drevni, mašte, rimljani, uzimao |
| 0682 | A blunt knife ruins a tomato salad | 3 | nož, nožem, stiže |
| 0683 | A letter from the tax office waits unopened | 2 | porez, poreza |
| 0684 | A child loses a tooth during lunch | 3 | odlučili, zuba, zubi |
| 0685 | A pupil remembers homework after leaving school | 2 | domaći, zadaću |
| 0686 | A camping story lasts longer than the fire | 4 | odavno, plamen, spavali, vatru |
| 0687 | A retiring librarian dislikes formal praise | 5 | dužnosti, oproštaj, prestaje, zahvalan, čast |
| 0688 | A muddy paw print appears on a new rug | 3 | nov, prešao, tepih |
| 0689 | A sketch looks better before the colour is added | 2 | crno, crtu |
| 0690 | A nephew makes his first solo bus journey | 3 | nećak, nećaka, viši |
| 0691 | A room colour changes after sunset | 3 | izgledali, zidove, zidovi |
| 0692 | A lakeside walk ends at the wrong picnic spot | 3 | dočekati, jezera, pošalju |
| 0693 | A haircut photo hides the back of the head | 2 | frizura, kosa |
| 0694 | A traveller boards a train going the other way | 2 | jugu, sjever |
| 0695 | A sour cup of tea gets a gentler second attempt | 4 | kapi, med, osjeti, tjera |
| 0696 | A path through the forest becomes quieter | 3 | drveće, nastavimo, šuma |
| 0697 | A portrait session moves closer to a window | 4 | izvana, kutu, svjetlosti, uhvatila |
| 0698 | A statue turns out to honour someone unexpected | 2 | kraljevi, kralju |
| 0699 | A first radio interview runs out of time | 4 | emisija, intervju, pokušam, snimka |
| 0700 | A bell tower tour pauses before the bells ring | 3 | razini, stoljeća, zvona |
