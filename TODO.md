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
* implement more fmt::Display cases for Instruction
* I'm not sure I'm actually "happy" with the generated Instruction enum?
  * it feels "rust-y"...but it's also clunky
  * ...and it's not that hard/expensive to just work with raw u32s?
* also dump symbol table in debug info?
  * maybe only if alternate is specified?
* instruction decoding should return an EmulatorError instead of String
