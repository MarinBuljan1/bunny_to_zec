# Historical conversation coverage review

This is the 700-to-653 cleanup snapshot. The library was subsequently expanded
back to 700 and renumbered consecutively at the user's request; see
[CONVERSATION_EXPANSION_REPORT.md](CONVERSATION_EXPANSION_REPORT.md) for that compaction.
Current totals are in [CONVERSATION_EXPANSION_1000_REPORT.md](CONVERSATION_EXPANSION_1000_REPORT.md).
IDs in the tables below refer to this historical snapshot, not the current library.

Coverage was audited across all 700 conversations. A **new form** means its first
occurrence in a learner reply while reading conversations in ascending permanent ID
order. This is an authoring audit, not the runtime review order and not a claim that
the word appears in only one conversation. Matching lowercases text while preserving
accents and grammatical forms, using the current app's token separators.
Partner text, English translations and metadata do not count.

## Result

- Before: **700 conversations; 4,663/8,574 forms covered**.
- Removed: **47 conversations** contributing no first appearances.
- Expanded: **all 59 conversations** originally contributing just one form, with
  individually written Croatian additions and matching contextual translations.
- Added: **87 forms previously absent from the entire library**.
- Lost coverage: **0 forms**.
- After: **653 conversations; 4,750/8,574 forms covered (55.4%)**.
- Still missing: **3,824 forms**. Full vocabulary coverage remains incomplete.
- Final audit: **no conversations contributing zero or one first appearances**.

Only those 59 dialogues were edited. The other retained dialogue records and the
vocabulary were preserved. Following partner turns were adjusted where needed to
answer added questions. Changes were checked for local context and contradictions,
including appointment duration and a photo walk's agreed meeting time.
This was a coverage review of the whole library and a language/flow review of the
targeted edits, not a fresh linguistic certification of every retained message.
Content remains a draft and the current review UI still uses existing sentences.
No other model or external API was used.

Index rows retain the compact `c_id` / `hits` format. Validation and a separate
recount checked occurrence counts, removed-ID references and coverage preservation.

## IDs at the time of this review

Surviving IDs were not renumbered in this cleanup, and removed IDs were initially
treated as retired. The later explicit request to fill the gaps superseded that
policy: the 653 survivors became `0001`–`0653`, and new conversations became
`0654`–`0700`. Match these historical entries by title rather than by their old ID.

## Removed conversations

Each row contributed zero first appearances in the original 700-conversation audit.
All its vocabulary forms remain in retained conversations.

| Removed ID in this snapshot | Situation |
| --- | --- |
| 0006 | Lunch at a restaurant: pasta |
| 0008 | Lunch at a restaurant: a vegetarian lunch |
| 0015 | Choosing clothes: a jumper |
| 0018 | A stop at the bakery: a cheese pie |
| 0019 | A stop at the bakery: pastries |
| 0020 | A stop at the bakery: a croissant |
| 0022 | Asking for directions: the museum |
| 0023 | Asking for directions: the beach |
| 0024 | Asking for directions: the post office |
| 0026 | Buying a bus ticket: Split |
| 0028 | Buying a bus ticket: Zadar |
| 0031 | Booking a room: a short break |
| 0038 | Arranging a repair: a cold radiator |
| 0052 | An evening at the cinema: a thriller |
| 0058 | Changing an appointment: waiting for a delivery |
| 0062 | Planning a work task: a presentation |
| 0066 | Studying with a classmate: biology |
| 0067 | Studying with a classmate: chemistry |
| 0068 | Studying with a classmate: geography |
| 0070 | Finding a book: gardening |
| 0071 | Finding a book: photography |
| 0072 | Finding a book: cooking |
| 0079 | Buying supplies at a pharmacy: bandages |
| 0080 | Buying supplies at a pharmacy: a hot-water bottle |
| 0082 | Starting an activity: yoga |
| 0083 | Starting an activity: walking |
| 0087 | Adjusting plans to the weather: a storm |
| 0096 | Looking for a lost item: a ticket |
| 0098 | Talking about a holiday: Pula |
| 0099 | Talking about a holiday: Osijek |
| 0100 | Talking about a holiday: Šibenik |
| 0392 | A robot speaks too quietly on stage |
| 0506 | Introducing a partner without making a speech |
| 0512 | A dog misunderstands a bird feeder |
| 0530 | A raincoat no longer fits a child |
| 0531 | A familiar voice over the supermarket speakers |
| 0568 | A grandmother’s recipe needs a video call |
| 0569 | A wall clock is blamed for being late |
| 0572 | A shoe repair ticket survives the wash |
| 0574 | A bakery saves the wrong pastry |
| 0575 | A photograph reverses a handwritten message |
| 0599 | A mirror makes the walk home unusual |
| 0600 | A child writes a whole letter without help |
| 0637 | A theatre prop sword needs repair |
| 0640 | An old map uses unfamiliar place names |
| 0697 | An uncertain banknote needs checking |
| 0700 | A neighbour finds a way to welcome newcomers |

## Expanded one-form conversations

The last column lists only globally new forms first introduced by that edit.
The final contribution can also include existing forms now appearing earlier in
the library. A form is counted only once in the 87-form total.

| ID | Situation | First appearances before | First appearances after | Globally new forms first added |
| --- | --- | ---: | ---: | --- |
| 0007 | Lunch at a restaurant: chicken | 1 | 9 | grah |
| 0010 | At the market: tomatoes and peppers | 1 | 10 | bocu, ulja |
| 0012 | At the market: strawberries and cherries | 1 | 10 | vrh, čokolada |
| 0014 | Choosing clothes: a jacket | 1 | 10 | crnom, crveni, išao |
| 0027 | Buying a bus ticket: Rijeka | 1 | 6 | darove, prtljažniku |
| 0030 | Booking a room: a weekend | 1 | 10 | boravak, posebni |
| 0044 | Returning a purchase: a broken lamp | 1 | 8 | prekidač, šteta |
| 0046 | Making dinner together: soup | 1 | 7 | podsjeti |
| 0047 | Making dinner together: rice | 1 | 7 | nađi |
| 0048 | Making dinner together: baked potatoes | 1 | 5 | prilog |
| 0050 | An evening at the cinema: an adventure film | 1 | 9 | glavobolju, uši |
| 0051 | An evening at the cinema: a documentary | 1 | 7 | pčele |
| 0054 | Arranging to meet: a bike ride | 1 | 5 | nagovoriti, odatle |
| 0055 | Arranging to meet: a picnic | 1 | 8 | ponesi, šah |
| 0056 | Arranging to meet: a photo walk | 1 | 9 | svjetlost, trava |
| 0060 | Changing an appointment: a family visit | 1 | 4 | ujak |
| 0074 | Booking a medical appointment: a cough | 1 | 4 | noćni |
| 0075 | Booking a medical appointment: an aching knee | 1 | 6 | ozlijedio |
| 0076 | Booking a medical appointment: an itchy rash | 1 | 5 | najgore, nastavim, nozi |
| 0078 | Buying supplies at a pharmacy: plasters | 1 | 3 | pecanje |
| 0084 | Starting an activity: dancing | 1 | 6 | partneri, paru, visine |
| 0088 | Adjusting plans to the weather: snow | 1 | 8 | zime |
| 0094 | Looking for a lost item: a phone | 1 | 4 | zovi, zvoni |
| 0305 | A banknote in a museum drawer | 1 | 3 | britanski, dolar |
| 0326 | A child dislikes a carnival costume | 1 | 2 | uplašen |
| 0378 | A parcel is heavier than expected | 1 | 2 | diplomu |
| 0381 | Glasses vanish before a class photograph | 1 | 3 | niz, zbunjen |
| 0382 | A bicycle needs its own place on the ferry | 1 | 2 | putnici |
| 0496 | A child finishes her first pool length | 1 | 3 | disanje |
| 0511 | Rain gets to a library book | 1 | 2 | slučajeve |
| 0522 | Choosing flowers for a family grave | 1 | 3 | groba, svijeće |
| 0526 | A café regular orders something different | 1 | 3 | ohladi, čašu |
| 0534 | A rehearsal accidentally invents a better ending | 1 | 3 | očekivali, smijeha |
| 0546 | A baby hat is smaller than expected | 1 | 2 | bebi |
| 0554 | A welcome cake reaches the wrong flat | 1 | 2 | pisalo |
| 0561 | A curtain becomes an accidental signal | 1 | 2 | budna |
| 0562 | A note in a lunchbox reaches the wrong sibling | 1 | 2 | kaznu |
| 0563 | A shopping list doubles as a bookmark | 1 | 2 | slovo |
| 0567 | Two buttons make a coat feel different | 1 | 2 | skladu |
| 0570 | A handwritten birthday letter arrives early | 1 | 2 | strategija |
| 0577 | Summer hours catch a reader out | 1 | 2 | posljednja |
| 0578 | A borrowed charger goes home in the wrong bag | 1 | 3 | None; adds earlier occurrences of existing forms |
| 0579 | A comic is read in the wrong order | 1 | 2 | traku |
| 0582 | A winter bag contains beach clothes | 1 | 2 | hladne |
| 0583 | A host checks a guest’s food preferences | 1 | 2 | prođe, stignu |
| 0584 | An imperfect vase finds another use | 1 | 2 | slomljena |
| 0588 | An inscription gives a second-hand book another story | 1 | 2 | list |
| 0589 | A trumpet rehearsal sounds like a car alarm | 1 | 2 | vježbe |
| 0590 | A seagull takes an interest in a picnic | 1 | 3 | plijen, zaklon |
| 0593 | A radio presenter forgets to start the song | 1 | 4 | papire, snimci, stvarnosti |
| 0596 | A bedtime story has an unexpected sound effect | 1 | 3 | protivnika, čaroliju |
| 0597 | A tiny café has room for one more chair | 1 | 2 | nosimo |
| 0618 | A stranger returns a lost wallet | 1 | 2 | umalo |
| 0632 | A film club debates an ambiguous ending | 1 | 2 | bitan |
| 0636 | A school trip crosses a border | 1 | 2 | signala |
| 0639 | A bicycle brake rubs after transport | 1 | 2 | odavdje |
| 0657 | A daughter asks about an old medal | 1 | 4 | jednoj, uniformu, vojnu |
| 0680 | A baker explains why the price changed | 1 | 2 | struju |
| 0687 | A flood warning changes a basement plan | 1 | 3 | rasti, razina |
