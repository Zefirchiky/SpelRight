# MangaHub SpellChecker

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

## Goals

- [x] Checking word correctness
- [x] Suggesting similar words
- [ ] Adding new words
- [x] Support different languages
- [ ] Make it fast

  Suggestions
  - [x] 100 words/s
  - [x] 250 words/s
  - [x] 1000 words/s
  - [x] 2500 words/s
  - [ ] 10000 words/s
  - [ ] 25000 words/s
  - [ ] 100000 words/s

  Loading
  - [x] <200 ms
  - [x] <100 ms
  - [x] <50 ms
  - [x] <20 ms
  - [x] <10 ms
  - [x] <5 ms
  - [x] <3 ms
  - [ ] <2 ms (read_to_string is more then 2 ms, not sure if even possible)

## Possible Optimizations

### Hardware

- [x] Cache locality (dence blob of words)
- [ ] SIMDeez nuts
  - [x] Distance finding
  - [ ] Binary search (might be optimized by the compiler)
- [ ] Parallelism
  - [ ] Rayon
    - [x] Test with and without
    - [ ] Auto desiding between parallel and normal
  - [ ] Manual
- [ ] GPU Acceleration

### Memory usage

- [x] Blob of words with no other symbold (aka. no `\n`)
- [x] Storing minimal offsets

Total memory usage is pretty much minimal.

### Reduce ammount of words checked

- [x] Word length groups (depend on dataset)
- [ ] For length that are max distance from a word (no chars change is allowed, only deletions)
  - [ ] Tracking first letter offsets, use only the once, whose first letter is the same
- [ ] For length that are the same as a word's (no chars deletion or insertion, only change)

### Caching

- [ ] Often mistakes

### Loading

> [!NOTE]
> read_to_string of 370000 words (~4 mb) is about 2 ms.
>
> **on my machine.**

- [x] Reduce parsing by pre-parsing the dataset, look `Better dataset`

### Better dataset

- [ ] Reduce words amount, most words are never used in an average text
- [x] Store offsets, no unnecessary `\n`
- [ ] Store first laters offsets

> [!NOTE]
> Will make it harder to work manualy with dataset.

### Better algorithms

- [x] Good algorithms for each LenGroup (from rapidfuzz)
- [ ] Custom
  - [ ] We know the word len, and currenly processing group's len. By this we can determine amount of deletions, insertions and changes. If done correctly, will be faster then any other algorithm. Example: word: `thks`, current group: 5, max change: 2. So there can only be 1 deletion, and 1 change for a word to be suggestion.
