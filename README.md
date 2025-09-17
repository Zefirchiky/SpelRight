# easy-spell-checker
A simple spell checker written in rust. Includes CLI and lib.

# Some benchmarks
Load and parse 4mb file with 370000 words in ~14ms.
Loads words in SpellChecker in ~25ms.
Words spelling check ~700µs
Sorted suggestions for 24 words against 370000 in ~11-13ms.

# CLI
spell.exe in %PATH%

```
> spell funny wrd sjdkfhsdjfh
Words loaded: 370105
✅ funny
❓ wrd => wro wry word wad rd wird ord urd ward wd
❌ Wrong word 'sjdkfhsdjfh', no suggestions
```