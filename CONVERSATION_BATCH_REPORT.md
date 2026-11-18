# Historical conversation batch: coverage audit

This is the snapshot before the subsequent coverage cleanup. The removed IDs
and expanded dialogues are recorded in
[CONVERSATION_REVIEW_REPORT.md](CONVERSATION_REVIEW_REPORT.md). A subsequent expansion
and requested ID compaction are recorded in
[CONVERSATION_EXPANSION_REPORT.md](CONVERSATION_EXPANSION_REPORT.md); IDs in this
historical batch table are not current references.
Current totals are in [CONVERSATION_EXPANSION_1000_REPORT.md](CONVERSATION_EXPANSION_1000_REPORT.md).

This is the first **100 conversations of the replacement 400-conversation expansion**.
The remaining **300 conversations are not complete**. The rejected draft of 400 close
variants was not added to the library. These 100 are individually written situations,
with five Croatian/English exchanges each, rather than four noun substitutions in a
shared dialogue template. No other model or external API was used.

Existing conversations `0001`–`0600` and all vocabulary entries were preserved.
New permanent IDs are `0601`–`0700`; the next ID is `0701`.

- Before: **600 conversations; 4,296 covered forms**.
- Added: **100 conversations; 367 distinct previously uncovered forms**.
- After: **700 conversations; 4663/8,574 forms covered (54.4%)**.
- Remaining vocabulary: **3911 forms**. Full coverage is incomplete.
- Measured gain: **3.67 new forms per conversation** across this batch.

The requested 1,000–3,000 additional forms across the full 400 remain a target,
not an achieved result. This first batch contributes 367 toward that target.
Its rate must not be assumed for the unwritten remainder.

Natural Croatian and faithful translations take precedence over coverage. There is
no minimum quota for individual conversations; four in this batch reinforce existing
forms without adding a new one. Similar broad topics may recur, but each conversation
has its own dialogue and situation. Coverage counts exact lowercased vocabulary forms
in learner replies only, preserving accents and grammatical distinctions. Partner
messages, English translations and metadata do not count. Repeated forms across this
batch are credited only once, to the first conversation introducing them.

The writer reviewed the dialogue and translations. Structural checks and an independent
recount validate the index and totals, not language quality. Content remains a draft
pending independent language review before use as finished teaching material.
The current review UI still uses the existing sentence content.

| Conversation ID | Situation | New forms | Forms first covered |
| --- | --- | ---: | --- |
| 0601 | A wheelchair route to the theatre | 3 | kolica, ovih, službenik |
| 0602 | A warning light on a borrowed car | 9 | auta, automobil, autu, crveno, pročitaj, razgovaram, riskirao, upalilo, zaustavimo |
| 0603 | Bread dough refuses to rise | 7 | izvrsno, naučim, nikako, posluži, rok, sira, strpljiv |
| 0604 | A library holds the wrong book | 6 | grešku, knjižnici, listu, posuditi, roman, trenutačno |
| 0605 | A choir rehearsal moves downstairs | 4 | pjevam, posljednju, sali, salu |
| 0606 | A bank card stays in the machine | 2 | objasnit, zadržao |
| 0607 | A family photograph needs permission | 2 | njihovi, odgovori |
| 0608 | A new employee asks about overtime | 4 | jasan, odobrenje, osnovne, poslovi |
| 0609 | A dog is afraid of a new lift | 4 | nježno, reagira, smiri, zvuka |
| 0610 | A washing label saves a wool jumper | 4 | djetetu, ručno, stupnjeva, vruća |
| 0611 | The wrong date on a concert ticket | 3 | isključivo, koncert, riješeno |
| 0612 | An envelope for the wrong apartment | 3 | nipošto, objasni, predaj |
| 0613 | Two people learn a board game | 5 | bacanje, igraj, pobjednik, skupi, usput |
| 0614 | An actor forgets which side to enter | 7 | glumi, izlazim, međusobno, predaje, princeza, sluga, uđe |
| 0615 | A child keeps a nature diary | 5 | detalji, lete, obliku, sjeme, temperaturu |
| 0616 | A swimmer misreads the lane sign | 7 | brze, disati, pauzu, pustim, ramena, sporo, srednja |
| 0617 | A market stall cannot take a large note | 3 | centi, dužan, pedeset |
| 0618 | A stranger returns a lost wallet | 1 | koži |
| 0619 | A street lamp shines into a bedroom | 4 | prolazi, službi, spavaću, zatražio |
| 0620 | A recipe needs less salt | 2 | izvadi, piletina |
| 0621 | A museum label hides a whole history | 4 | civilizacije, metala, muzeju, smatraju |
| 0622 | An interview recording needs consent | 2 | izgledao, svako |
| 0623 | A patient prepares questions for a doctor | 6 | bol, dozu, lijeka, najvažnija, simptome, zapamti |
| 0624 | An office printer exposes a private document | 5 | dokument, klijenta, licem, pomoću, sustav |
| 0625 | A science club compares two samples | 2 | količina, kontrola |
| 0626 | A local paper corrects a quotation | 2 | ispada, projektu |
| 0627 | A ferry passenger packs the wrong bag | 3 | brodom, izdržati, otok |
| 0628 | A flatmate asks for quiet at night | 4 | dijelimo, odsad, prenosi, uzrujana |
| 0629 | A pensioner learns to enlarge text | 2 | korake, veličina |
| 0630 | A community garden shares its harvest | 3 | jednostavna, nemojmo, skupina |
| 0631 | A beginner joins a horse-riding lesson | 3 | jahati, konja, čvrst |
| 0632 | A film club debates an ambiguous ending | 1 | skrivao |
| 0633 | A charity explains its expenses | 5 | aktivnosti, godišnje, izvješće, reklame, sredstava |
| 0634 | A witness remembers less than expected | 2 | došlo, pad |
| 0635 | A small business outgrows its spreadsheet | 5 | dupli, izbrisati, klijenata, odrediti, trošiti |
| 0636 | A school trip crosses a border | 1 | granici |
| 0637 | A theatre prop sword needs repair | 0 | None; reinforces previously covered forms |
| 0638 | A student asks for a quieter exam room | 4 | ljubazan, uvjetima, velikoj, zahtjeva |
| 0639 | A bicycle brake rubs after transport | 1 | kočnice |
| 0640 | An old map uses unfamiliar place names | 0 | None; reinforces previously covered forms |
| 0641 | A visitor waits at the wrong hospital entrance | 6 | bolnica, bolnici, leži, liječnik, posjeti, čistu |
| 0642 | A pharmacist checks an unclear label | 3 | lijekova, lijekove, uzimati |
| 0643 | A church choir welcomes a nonreligious visitor | 5 | cijeniti, crkve, crkvi, crkvu, čine |
| 0644 | A tenant speaks to a lawyer | 4 | odvjetnik, odvjetnikom, oštećenja, pregovarati |
| 0645 | A film subtitle changes a police rank | 4 | izdaje, policijski, poručnik, poručniče |
| 0646 | An astronomy class waits for a clear sky | 4 | galaksije, kreće, približi, satelit |
| 0647 | A reunion invitation uses an old surname | 3 | braka, njezinim, razvod |
| 0648 | A campsite asks visitors to separate waste | 3 | kampu, otpada, vrste |
| 0649 | A radio host receives a rude message | 5 | emisiji, gosta, nastavi, reakciju, uvrede |
| 0650 | A family learns what a DNA kit can reveal | 6 | dnk, genetski, nudi, odnose, tvrtka, čuvaju |
| 0651 | A driver refuses another drink | 6 | alkohol, plesa, uzmimo, vozim, vožnji, žedan |
| 0652 | A climber turns back before the summit | 8 | bole, jači, koljena, nikamo, staze, stijene, ugroziti, uvjeti |
| 0653 | A roof leak reaches a neighbour | 6 | krova, mrlje, odmakni, osiguranja, smjeli, upravitelj |
| 0654 | A child is disappointed by a lost race | 7 | brži, kažeš, rekord, snagu, tjednima, utrke, utrku |
| 0655 | An artist explains an empty frame | 4 | djelu, prazno, umjetnik, čiji |
| 0656 | A dentist asks about a damaged tooth | 3 | jeo, straga, zub |
| 0657 | A daughter asks about an old medal | 1 | vojsci |
| 0658 | A café corrects an allergy misunderstanding | 2 | kikiriki, žuri |
| 0659 | A beginner hears the difference between two notes | 3 | gitaru, tona, žicu |
| 0660 | A visitor asks about a synagogue tour | 5 | obilazak, posjeta, sigurnosne, susjedstvu, židovi |
| 0661 | A friend is angry about a broken promise | 4 | bijesan, grubo, izluđuje, zraka |
| 0662 | A new resident feels lonely in a busy city | 5 | klubu, pridružiti, stranac, tečaj, usamljena |
| 0663 | A wedding invitation needs a tactful refusal | 2 | putujem, sreća |
| 0664 | A promotion brings unexpected doubts | 3 | sumnjam, tima, upravljanje |
| 0665 | A child admits breaking a window | 2 | stakla, uplašio |
| 0666 | A bad review feels personal | 2 | stresom, čekanje |
| 0667 | A father forgets a school performance | 3 | nastup, plakala, tužna |
| 0668 | A friend keeps a surprising secret | 5 | dosadna, iskren, iznova, ostaviš, stigneš |
| 0669 | A friend asks before giving advice | 3 | ideje, ispričaj, poželiš |
| 0670 | A couple disagrees about moving | 5 | obećanja, odlučila, opcije, preseliti, slažemo |
| 0671 | A crime novel names the wrong suspect | 4 | optužba, pokušavao, sina, svjedoka |
| 0672 | A local election discussion stays civil | 4 | dodatno, mišljenju, planiraju, stranke |
| 0673 | A factory tour explains protective equipment | 6 | diljem, napusti, proizvode, tvornicu, zraku, šalju |
| 0674 | A weather report cancels a boat trip | 3 | sigurnost, situaciju, stranici |
| 0675 | A city removes a dangerous tree | 9 | blok, duboko, iznutra, padaju, prikladno, stručnjak, tlo, ući, zelena |
| 0676 | A customer questions a repair estimate | 2 | isteklo, uključen |
| 0677 | A child wants to keep a wild animal | 3 | nauditi, službu, životinju |
| 0678 | A listener spots a misleading advertisement | 6 | bijelom, dokaze, navodi, pokazuju, ponavlja, učinak |
| 0679 | A parent learns a child has been bullied | 2 | dječaci, tjedana |
| 0680 | A baker explains why the price changed | 1 | računam |
| 0681 | A suitcase arrives on a later flight | 5 | aerodromu, avionom, majicu, svađa, zube |
| 0682 | A handwritten recipe names no quantities | 4 | mlijeka, osjećaju, palačinke, vrijede |
| 0683 | A volunteer learns to walk shelter dogs | 5 | naglo, psi, tvojem, upozorenje, zaposlenika |
| 0684 | A student receives a scholarship offer | 4 | mjesečno, smanjiti, smjena, sveučilište |
| 0685 | A chess player resigns too early | 3 | napasti, osvojiti, šanse |
| 0686 | A tailor asks a customer to move | 6 | nazovite, odijelu, sjedite, udobno, završeno, četvrtak |
| 0687 | A flood warning changes a basement plan | 1 | podrumu |
| 0688 | A dancer chooses a simpler routine | 3 | nadzor, odustajem, skok |
| 0689 | A clock repairer asks for patience | 2 | dnevno, financijski |
| 0690 | A picnic guest admits disliking a dish | 3 | jedem, obožavam, omiljena |
| 0691 | A night worker negotiates household noise | 3 | paziš, spavam, spavanje |
| 0692 | A visitor mistakes an animal rescue for a zoo | 4 | kućni, utočište, uče, čuvari |
| 0693 | A friend prepares for a funeral | 3 | kasnimo, petnaest, pogreb |
| 0694 | A child asks why a fairy tale is frightening | 5 | postavlja, slijedi, stvara, tama, čin |
| 0695 | An old friend returns from abroad | 3 | planova, stekao, velikih |
| 0696 | A student changes a presentation after feedback | 5 | glavne, objašnjava, svrhe, točke, zaustavi |
| 0697 | An uncertain banknote needs checking | 0 | None; reinforces previously covered forms |
| 0698 | A birthday surprise clashes with a quiet preference | 3 | shvatila, središtu, zamisao |
| 0699 | A couple chooses a wedding photographer | 3 | stvarne, uzmemo, zore |
| 0700 | A neighbour finds a way to welcome newcomers | 0 | None; reinforces previously covered forms |
