# mangahub-spellchecker

A simple spell checker written in rust. Includes CLI and lib.

Supports any utf-8, as long as input file is of right format (look SpellChecker::new or load_words_dict).

Was primeraly writen for MangaHub project's Novel ecosistem. And to learn Rust :D

## Some benchmarks

Load and parse 4mb file with 370105 words in ~6ms.

Words spelling check ~13,000,000 words/s for all incorrect words (worst case scenario).

Sorted suggestions for 30 incorrect words in ~10ms (3000 words/s).

Memory usage is minimal, one big string of all words without a dilimeters + a small vec of information.
Totaling dict size + ~200 bytes (depending on the biggest word's length) + additional cost of some operations.

## CLI

`spell.exe` in %PATH%. `words.txt` in the same folder.

```shell
> spell funny wrd sjdkfhsdjfh
✅ funny
❓ wrd => wro wry word wad rd wird ord urd ward wd
❌ Wrong word 'sjdkfhsdjfh', no suggestions
```

## Possible Optimizations

### Hardware

[x] Cache locality (dence blob of words)

[ ] SIMDeez nuts
    [x] Distance finding
    [ ] Binary search (might be optimized by the compiler)

[ ] Parallelism
    [ ] Rayon
        [x] Test with and without
        [ ] Auto desiding between parallel and normal
    [ ] Manual

### Memory usage

[x] Blob of words with no other symbold (aka. no `\n`)

[x] Storing minimal offsets

Total memory usage is pretty much minimal.

### Reduce ammount of words checked

[x] Word length groups (depend on dataset)

[ ] For length that are max distance from a word (no chars change is allowed, only deletions)
    [ ] Tracking first letter offsets, use only the once, whose first letter is the same

### Caching

[ ] Often mistakes

### Loading

> [!NOTE]
> read_to_string of 370000 words (~4 mb) is about 2 ms.
>
> **on my machine.**

[ ] Reduce parsing by pre-parsing the dataset, look `Better dataset`

### Better dataset

[ ] Reduce words ammount, most words are never used in an average text

[ ] Store offsets, no unnecessary `\n`
> [!NOTE]
> Will make it harder to work manualy with dataset.
