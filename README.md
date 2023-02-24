# Pokémon Sprite Scrape
A small utility program I wrote to help download Pokémon sprites for my [Champs in the Making website](https://github.com/oneirocosm/champs-frontend).

This small program uses asynchronous Rust to download multiple sprite files at a time.  It works by finding the names of various pokemon using [my database](https://github.com/oneirocosm/champs-db) and then combining them with a few common URLs to download the files from [PokémonDB](https://pokemondb.net/).  It is set up in such a way that it will attempt to find a [Scarlet / Violet](https://scarletviolet.pokemon.com/en-us/) sprite first.  If it can't find that, it will fall back to a [Brilliant Diamond / Shining Pearl](https://diamondpearl.pokemon.com/en-us/) sprite.  And if it can't find that, it will fall back to a [Pokémon Home](https://home.pokemon.com/en-us/) (used to be called Pokémon Bank) sprite.

The program is currently hard-coded to only do one thing, but it could easily be extended to allow for custom user input if necessary.

## How to Run
1. Set up Rust using [rustup](https://rustup.rs/) or update if it's already installed
1. Download this repository
1. Run the command `cargo run`

## Observations from Building This
This program ended up being much more interesting than I expected.  While I have done some async development in the past, this is the first time I really got to use Rust for it.  I really love how the language uses Lazy Futures instead of Eager ones.  For some reason, that feels more intuitive to me.

The way the Rust async ecosystem is split into a bunch of competing crates is a little frustrating, but it makes sense given that it's still kind of an experimental area.  I particularly liked that the futures crate had a `for_each_coconcurrent` method as a part of their implementation of `StreamExt`.  I didn't completely understand why the program failed when I didn't specify a hard number for the number of futures allowed to run at once&mdash;I suppose that aspect of it hasn't been sorted out yet.  It is a little weird to use a number that feels arbitrary there&mdash;I just sort of picked something that gave me good performance without causing problems.
