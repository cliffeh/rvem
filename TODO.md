* refactor to make rvem a subproject (rename repo 'rv'?)
* test coverage for individual instructions?
* precommit hook for beautification (cargo fmt)
* github CI workflows for build/test
  * this could be tough (would require risc-v tools)
    * maybe check in binaries?
* assembler?
* handle errors for things like:
  * trying to write to .text
  * attempt to divide by zero
* commit a gcc cross-compiled binary for testing
