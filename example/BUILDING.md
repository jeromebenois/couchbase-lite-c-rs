
Requirements
------------

- Rust + Cargo (unknown version but probably whatever stable is, at least ed. 2018)
- Rustfmt

For those, use rustup to install.

From the `couchbase-lite-c` project :

* GCC 7+ or Clang
* CMake 3.9+
* ICU libraries (`apt-get install icu-dev`)

Note : for me it was `apt-get install libicu-dev`

I also needed to add

* `zlib1g-dev`

Python and Perl are also searched but are probably optional (needed to build language bindings).

To use bindgen, we need Clang, so actually GCC is not preferred...

Also needed: llvm.