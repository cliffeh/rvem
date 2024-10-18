* refactor to make rvem a subproject (rename repo 'rv'?)
* cleanup (god there's some awful shit here)
* rectify RISC-V+Linux vs. MIPS syscalls
* go for RV32m (multiply) extensions?
  * maybe with separate feature flags/targets for what it supports
* test coverage for individual instructions?
* beautifier for asm files?
* precommit hook for beautification
* github CI workflows for build/test
  * this could be tough (would require risc-v tools)
    * maybe check in binaries?
* re-organize code so the examples aren't in the main project dir?
* the Makefile is nice - is there a more "rust-y" way to do it?
  * ...and do I give a fuck?
* fix whatever is broken with jalr/unsigned addition?
  * make fib work ffs

