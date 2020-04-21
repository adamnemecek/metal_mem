# TODO

* [x] byte_capacity and capacity are mixed up because when we allocate whole pages but only uses a subst of these
* [ ] fix splice_slow
    * 
* [ ] for v2, maybe i can just use allocref
* [ ] try to get rid of the copy requirement on gpuvec

* [ ] create your own allocref trait
* [ ] add trait implementations from https://doc.rust-lang.org/std/vec/struct.Vec.html
* [x] fix iterators
* [ ] drain
* [ ] drain filter

* [x] debug_tuple
* [ ] truncate

* [ ] deref makes vec behave like a slice
* [ ] dedup dedupbykey
* [x] append
* [ ] in drain, what is T: Copy + 'a
* [x] doubleendediterator

## GPUVar
* [ ] use memalign like in vec
* [ ] use 