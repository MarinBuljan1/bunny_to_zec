# Conversation expansion: 800 to 900

This is a historical batch. The subsequent expansion to 1,000 is recorded in
[CONVERSATION_EXPANSION_1000_REPORT.md](CONVERSATION_EXPANSION_1000_REPORT.md).

This batch adds **100 directly authored conversations**, IDs **`0801`–`0900`**.
Each has five complete Croatian/English exchanges. No external API or other model
was used. Dialogues were written individually, without noun-substitution templates.

## Coverage

- Before: **800 conversations; 5,166/8,574 forms covered**.
- Added: **100 conversations; 261 distinct previously uncovered forms**.
- Average: **2.61 new forms per added conversation**.
- After: **900 conversations; 5427/8,574 forms covered (63.3%)**.
- Still missing: **3147 forms**. Full coverage remains incomplete.
- Every new conversation adds **at least two previously uncovered forms**; no exception was needed.
- The original 800 records, including IDs, dialogue, translations, titles and order, are **unchanged**.
- No old vocabulary coverage was lost. All 900 conversations still contribute at
  least two first appearances in ID order.

Counts use exact lowercased vocabulary forms in learner replies only. Diacritics
and grammatical distinctions remain separate. Each repeated new form is credited
only to the first conversation introducing it. Partner text, English translations,
titles and metadata do not count.

Natural Croatian takes priority over density. Review included grammatical agreement,
dialogue continuity, and contextual translations. Wording was simplified where it
felt strained, even when that removed a potential new form. The corpus remains a
draft pending independent language review; automated checks do not certify grammar.

## Format and validation

IDs remain consecutive **`0001`–`0900`**. The next new ID is **`0901`**. Index rows
retain **`c_id` / `hits`**. The three approved sample IDs (`0070`, `0071`, `0072`)
and their occurrence weights remain unchanged. The current review UI still uses
existing sentence content.

The validator checks unique IDs, duplicate complete dialogues, 5–8 exchanges,
bilingual text and index consistency. A separate recount checked every selection
weight, contribution counts and old coverage preservation. New Croatian messages
were also checked for accidental Cyrillic or replacement characters.

## Added conversations

| ID | Situation | New forms | Forms first covered |
| --- | --- | ---: | --- |
| 0801 | A small workshop hires its first employee | 3 | zaposlen, zaposlio, zaposliti |
| 0802 | A student weighs two different placements | 3 | karijere, karijeri, karijeru |
| 0803 | An amateur film gets a borrowed camera | 2 | producent, profesionalno |
| 0804 | A river sample arrives without its location label | 2 | analizu, testove |
| 0805 | A historical map changes between two pages | 2 | carstvo, drevnim |
| 0806 | A new librarian learns the return machine | 3 | korištenje, obuku, otkrijem |
| 0807 | A night crossing is somebody’s first cabin journey | 4 | kabini, putnik, putovanju, smetam |
| 0808 | A piano student compares recordings a year apart | 4 | današnje, raste, sposobnosti, vještina |
| 0809 | A social club changes its name on a sign | 2 | zajednica, zajednice |
| 0810 | A village bus route is shortened on a map | 3 | liniju, mnogim, veza |
| 0811 | An old prison cell is smaller than a visitor expected | 2 | ćelija, ćelije |
| 0812 | A tennis beginner counts mistakes instead of points | 2 | loših, udaraca |
| 0813 | A mine tour has an optional deeper section | 2 | dublje, rudnik |
| 0814 | A friend waits at the wrong entrance to a station | 2 | zakasniti, zapadu |
| 0815 | Two siblings remember disliking the same lunch | 3 | mrzila, mrzio, činjenica |
| 0816 | A passport stamp belongs to a vanished country | 4 | država, državi, državu, putovao |
| 0817 | A child learns why two farm animals stay apart | 3 | bik, domaćin, krava |
| 0818 | A spelling medal feels smaller than the effort behind it | 2 | medalju, osvojio |
| 0819 | A friend respects a work story that cannot be shared | 2 | povjerljivo, tajnosti |
| 0820 | A technical explanation becomes clear after one drawing | 2 | jezikom, tehnički |
| 0821 | A school alarm sounds during a quiet lesson | 2 | sirena, uzbuna |
| 0822 | A narrow path makes two walkers take turns | 3 | oprezan, oprezno, smjera |
| 0823 | A novelist spots her own book in a shop window | 4 | izdao, objavio, slikati, zbunjena |
| 0824 | A gold-coloured medal is not made of gold | 3 | sjaj, zlata, zlatni |
| 0825 | A posted document has no tracking update yet | 3 | poslan, potvrdio, praćenje |
| 0826 | A lecture recording reveals a missed qualification | 2 | predavanje, profesora |
| 0827 | An apology names the hurt without arguing about intent | 4 | osjećao, povrijedila, slušaš, uvrijedio |
| 0828 | A child volunteers to prepare the painting table | 3 | napuni, operi, vodom |
| 0829 | A visitor follows the old city centre on a new map | 2 | karta, središte |
| 0830 | A family event poster forgets the practical details | 3 | nigdje, obiteljima, obiteljski |
| 0831 | A second-hand shop separates donated clothes from stock | 3 | pišu, roba, robe |
| 0832 | A camp pickup needs the right names | 2 | dvojice, dvojicu |
| 0833 | A postcard makes a distant relative feel nearer | 3 | americi, amerika, očima |
| 0834 | An old letter contains two languages | 4 | njemačkoj, pisao, ruski, živio |
| 0835 | A menu translation turns a dish into a strange sentence | 3 | doslovno, kineski, tehnologija |
| 0836 | A heavy bag stays on the same shoulder | 2 | lijevom, rame |
| 0837 | A new driver volunteers for a familiar route | 3 | sjedalo, vozačku, vozimo |
| 0838 | A dancer needs the starting point marked | 3 | korak, slijediti, zadnjem |
| 0839 | A fundraiser chooses one person to answer questions | 2 | dajem, izjave |
| 0840 | A Sunday pie recalls a grandmother’s kitchen | 2 | peče, pite |
| 0841 | A shared dog walk needs more than one lead | 4 | pasa, psima, psu, umire |
| 0842 | A place card uses only one person’s surname | 2 | suprug, suprugu |
| 0843 | A workshop cleanup uses the last clean cloth | 2 | prljava, prljavi |
| 0844 | A slow queue is longer than it first looked | 3 | brzinom, stotina, vraćaš |
| 0845 | A classroom skeleton has a loose rib | 4 | kost, kostiju, rebra, učitelja |
| 0846 | A game guard forgets to watch the gate | 4 | postavio, stražu, zalihe, čuvara |
| 0847 | A writer cannot decide what is behind a door | 2 | maštu, mislima |
| 0848 | A kitchen radio finds a station from childhood | 2 | radija, radiju |
| 0849 | A team captain is not simply the top scorer | 2 | igrača, najboljeg |
| 0850 | A plant support bends under new growth | 2 | štap, žica |
| 0851 | A club chair learns where her authority ends | 2 | autoritet, ovlasti |
| 0852 | A daughter realises her mother kept every postcard | 2 | kćeri, majkom |
| 0853 | Two children admit breaking different parts of a model | 2 | slomila, slomio |
| 0854 | A picnic cooler becomes a seat by accident | 2 | genij, genijalno |
| 0855 | A loose button delays someone at the door | 2 | iglu, čvrsto |
| 0856 | A film sound effect wakes a sleeping viewer | 3 | eksplozija, eksplozije, ostaješ |
| 0857 | Two editions give a fairy-tale wizard different names | 2 | čarobnjak, čarolija |
| 0858 | A sociable friend admits feeling lonely at weekends | 3 | mislimo, usamljen, zauzet |
| 0859 | A family invitation lists someone’s new partner incorrectly | 5 | njihovim, odabrali, oženjen, rođak, zaručnica |
| 0860 | A group video includes someone who did not agree to filming | 2 | hitno, snimili |
| 0861 | A compass stops a group arguing about a map | 3 | jug, sjeveru, stali |
| 0862 | A brightly coloured drink turns out to be alcohol-free | 3 | alkohola, koktel, soka |
| 0863 | A temperature display cannot decide which side of zero to show | 2 | nula, stupanj |
| 0864 | A first payslip makes a planned purchase feel different | 2 | plaću, zaradio |
| 0865 | A choir archive needs names, not just a group label | 2 | ženama, ženski |
| 0866 | A tiny rustle interrupts a quiet evening | 3 | miš, nikakvog, traga |
| 0867 | A volunteer accepts a different task after declining one | 2 | odbila, pristala |
| 0868 | A train journey becomes a game of spotting landmarks | 2 | dosadan, dosadno |
| 0869 | A dinner portion is smaller than the menu photograph | 2 | komadić, odrezak |
| 0870 | A stage illusion works differently from the side seats | 3 | magija, magije, pustit |
| 0871 | A frown is mistaken for anger | 3 | ljuti, ljutiš, peti |
| 0872 | A wet poster is damaged but not lost | 3 | pokušamo, uništen, uništili |
| 0873 | A paper frog jumps farther than its maker expects | 2 | skoči, skočiti |
| 0874 | An exhibition invitation is read one day too early | 2 | otvaranje, zatvoreno |
| 0875 | A riverside path changes sides at a new footbridge | 2 | pravcu, prelazi |
| 0876 | A shopping list gets shorter after the cupboard is checked | 2 | kupnju, namirnice |
| 0877 | A sister’s first story collection contains familiar places | 2 | izdala, tajne |
| 0878 | A meeting invitation makes the purpose clearer than the agenda | 4 | prisutan, suradnju, svrhu, upoznate |
| 0879 | An imperfect photograph becomes a treasured family picture | 2 | ljepotu, vrijedna |
| 0880 | A rented van has an earlier return time than expected | 2 | ugovora, unajmio |
| 0881 | A narrator loses her place when a stage entrance changes | 2 | izvodi, narator |
| 0882 | A family uniform needs a more careful label | 3 | mornarice, potvrđeno, vojna |
| 0883 | A parent’s career change looks different in hindsight | 3 | živimo, životima, životni |
| 0884 | An airport shuttle leaves someone’s bag at the wrong door | 3 | aerodroma, aviona, predao |
| 0885 | An invented creature gets a name after it is finished | 2 | stvorenja, stvorenje |
| 0886 | Flat-pack instructions make more sense from the other side | 2 | muka, muke |
| 0887 | A surprise party nearly gives itself away | 2 | budimo, mirni |
| 0888 | A familiar smell changes an old memory | 4 | drugoga, jednoga, sjećanja, sjećanje |
| 0889 | A magnet demonstration has no visible rope | 3 | sila, sile, uže |
| 0890 | A stage fight looks convincing only at the right speed | 3 | borba, borbu, punom |
| 0891 | Green curtains look different in a bright room | 2 | svijetla, zelene |
| 0892 | A canal drawing needs a bridge in the right place | 2 | kanal, kanala |
| 0893 | A comic-book cover hides the supporting characters | 3 | heroj, heroji, razgovore |
| 0894 | A report’s missing page explains a surprising total | 2 | izvješća, podacima |
| 0895 | A paper dragon runs out of one colour | 2 | crvenog, žuti |
| 0896 | A bookseller cannot promise an out-of-print edition | 4 | spomenuli, tražite, trgovac, tržištu |
| 0897 | A headteacher records the wrong announcement version | 2 | najvažniji, ravnatelj |
| 0898 | A forgotten bin explains a smell in a clean kitchen | 3 | odvratno, smrad, vreću |
| 0899 | An improvised outing beats a cancelled plan | 4 | nijedna, sjajan, sjajne, uhvatili |
| 0900 | A weak phone signal cuts a conversation short | 2 | slab, slaba |
