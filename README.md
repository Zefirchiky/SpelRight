# mangahub-spellchecker
A simple spell checker written in rust. Includes CLI and lib.

Supports any utf-8, as long as input file is of right format (look SpellChecker::new or load_words_dict).

Was primeraly writen for MangaHub project's Novel ecosistem. And to learn Rust :D

# Some benchmarks
Load and parse 4mb file with 370105 words in ~6ms.

Words spelling check ~13,000,000 words/s for all incorrect words (worst case scenario).

Sorted suggestions for 30 incorrect words in ~16-17ms (1600 words/s).

# CLI
`spell.exe` in %PATH%. `words.txt` in the same folder.

```
> spell funny wrd sjdkfhsdjfh
✅ funny
❓ wrd => wro wry word wad rd wird ord urd ward wd
❌ Wrong word 'sjdkfhsdjfh', no suggestions
```