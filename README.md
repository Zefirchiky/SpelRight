# MangaHub SpellChecker (Name will be changed)

A simple spell checker written in Rust. Includes CLI and lib.

Supports any utf-8, as long as input file is of right format (look [Dataset Fixer](https://github.com/Zefirchiky/easy-spell-checker/tree/ca505359efdc0a862d3418ae3c8b9f0418a9f25e/dataset_fixer) or load_words_dict).

Was primirely written for [MangaHub](https://github.com/Zefirchiky/MangaHub) project's Novel ecosistem. And to learn Rust :D

## Some benchmarks

On my i5-12450H laptop with VSC opened.

Load and parse 4mb file with 370105 words in ~2ms.

Words spelling check ~45,000,000 words/s for all correct words (worst case scenario, batch_par_check).

Sorted suggestions for 1000 incorrect words in ~110ms (~9000 words/s, batch_par_suggest).

Memory usage is minimal, one big string of all words without a delimiters + a small vec of information.
Totaling dict size + ~200 bytes (depending on the biggest word's length) + additional cost of some operations.

## CLI

`spell.exe` in %PATH%. `words.txt` in the same folder.

```shell
> spell funny wrd sjdkfhsdjfh
✅ funny
❓ wrd => wro wry word wad rd wird ord urd ward wd
❌ Wrong word 'sjdkfhsdjfh', no suggestions
```

## Breakthroughs that lead to this

### Storing blobs of words, and their metadata

Storing words of each length in immutable (optional) blobs, sorted by bytes.

Store info about those blobs: len and/or count.

Pros:

- Incredibly easy to iterate over
- SIMD compatible
- Hightly parallelizable
- Great cache locality (a shit ton of cache hits)
- Search words with binary search `O(log n)`
- Working with bytes instead of chars
  - Support any language
- Other that I forgor

Cons:

- Needs precise dataset
- Pretty difficult words addition without moving the whole Vec

Pros totally outwheight the Cons!

### Specialized matching algorithm

When iterating over each LenGroup, based on max difference, we can calculate maximum amount of deletions, insertions and changes.

As an example:

Cheking `nothng` (group 6) against group 7, the differens between them is 1 insertion and 1 (optional) change.

With one insertion, `nothng` will become group 7, and with optional change it can match other words.

There will always be exactly `max_dif` of `max_delete + max_insert + max_change`.

This is multiple times faster then any other distance finding algorithm.

## Goals

- [x] Checking word correctness
- [x] Suggesting similar words
- [ ] Adding new words
- [x] Support different languages
- [ ] Make it fast
- [ ] Make good CLI
  - [ ] Long runing Server
  - [ ] Config

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
  - [x] Distance matching
  - [ ] Binary search (might be optimized by the compiler)
- [ ] Parallelism
  - [ ] Rayon
    - [x] Test with and without
    - [ ] Auto desiding between parallel and normal
  - [ ] Manual
- [ ] GPU Acceleration

### Memory usage

- [x] Blobs of words with no other symbold (aka. no `\n`)
- [x] Storing minimal metadata about each word length
- [ ] Storing first letter offsets, size depends on the language, but minimal overall

Total memory usage is pretty much minimal.

### Reduce ammount of words checked

- [x] Word length groups (depend on dataset)
- [ ] For length that are max distance from a word (no chars change is allowed, only deletions)
  - [ ] Tracking first letter offsets, use only the once, whose first letter is the same
- [x] For length that are the same as a word's (no chars deletion or insertion, only change)

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
> Made it harder to work manualy with dataset.

### Better algorithms

- [x] Custom
  - [x] See Breakthrough
