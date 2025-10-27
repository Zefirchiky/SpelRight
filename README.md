# Spel Right

Yes, it is intentional.

A simple Spell Checker written in Rust. Includes CLI and lib.

Also avaliable in [crates.io](https://crates.io/crates/mangahub-spellchecker)!

Supports any utf-8, as long as input file is of right format (look [Dataset Fixer](https://github.com/Zefirchiky/SpelRight/blob/49247d1db4ad47746484e1cdd809b7bdec336ffe/dataset_fixer/src/main.rs) or [load_words_dict](https://github.com/Zefirchiky/SpelRight/blob/49247d1db4ad47746484e1cdd809b7bdec336ffe/src/load_dict.rs)).

Was primirely written for [MangaHub](https://github.com/Zefirchiky/MangaHub) project's Novel ecosistem. And to learn Rust :D

## Some benchmarks

On my i5-12450H laptop with VSC opened.

Load and parse 4mb file with 370105 words in ~<2ms.

Words spelling check ~50,000,000 words/s for all correct words (worst case scenario, `batch_par_check`).

Sorted suggestions for 1000 incorrect words in ~80ms (~12500 words/s, words case scenario, `batch_par_suggest`).

Memory usage is minimal, a few big strings of all words without a delimiters + a small vec of information.
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

When iterating over each `LenGroup`, based on `max difference`, we can calculate maximum amount of `deletions`, `insertions` and `substitutions`.

As an example:

Cheking `nothng` (group 6) against group 7, the differens between them is 1 `insertion` and 1 (optional) `substitution`.

With one insertion, `nothng` will become group 7, and with optional `substitution` it can match other words.

There will always be exactly `max_dif` of `max_delete + max_insert + max_substitution`.

This is **multiple times** faster then any other distance finding algorithm.

## Goals

- [x] Checking word correctness
- [x] Suggesting similar words
- [ ] Adding new words
- [x] Support different languages
- [ ] Full languages support
  - [x] Full askii support
  - [ ] Full UTF-8 support
    - [ ] Normalize some languages
    - [ ] Divide languages into words with pure askii, with possible normalization, and with present UTF-8
  - [ ] Plugin
    - [ ] For everything
      - [ ] Default plugins
    - [ ] For especially complex languages
- [ ] Make good CLI
  - [ ] Long runing Server
  - [ ] Config
- [ ] Make it fast

  Suggestions (12500 words/s)
  - [x] 100 words/s
  - [x] 250 words/s
  - [x] 1000 words/s
  - [x] 2500 words/s
  - [x] 10000 words/s
  - [ ] 25000 words/s
  - [ ] 100000 words/s

  Loading (2.2 ms)
  - [x] <200 ms
  - [x] <100 ms
  - [x] <50 ms
  - [x] <20 ms
  - [x] <10 ms
  - [x] <5 ms
  - [x] <3 ms
  - [x] <2 ms (read_to_string is more then 2 ms, not sure if even possible (nvm, after reloading pc, its less then 2 ms))
  - [ ] <1 ms (No idea how the fuck this could be possible, but hey, goals!)

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
