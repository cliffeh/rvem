* refactor to make rvem a subproject (rename repo 'rv'?)
* cleanup (god there's some awful shit here)
* test coverage for individual instructions?
* precommit hook for beautification
* github CI workflows for build/test
  * this could be tough (would require risc-v tools)
    * maybe check in binaries?
* the Makefile is nice - is there a more "rust-y" way to do it?
  * ...and do I give a fuck?
* assembler?
* handle errors for things like:
  * trying to write to .text
  * attempt to divide by zero

