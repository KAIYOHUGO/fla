# FLA

Just a simple tool for making a lot of flash card in [logseq](https://logseq.com/)

```
apple {
    n {
        蘋果
    }
    just some text
}
```

`n` is the speech, it can be `n`, `v`, `o`, `adj`, `adv`, `prep`, `pron`

```bash
$ fla -h
Usage: fla [OPTIONS] <MODE> <INPUT>

Arguments:
  <MODE>   [possible values: build, debug, fmt]
  <INPUT>  

Options:
  -o, --output <OUTPUT>  
  -h, --help             Print help information
  -V, --version          Print version information
```
