## Introduction

MoveAssembler is a tool (yet another but better one resembles [movetool](https://github.com/Zellic/movetool)) that allows disassembling Move module/script binary files and reassembling them back into the binary format. It also supports verifying the Move binary.


## Building

## Example usage

```shell
# Build the CLI tool
$ cargo build

# Display help information for the MoveAssembler tool
$ ./target/debug/move-assembler --help
Usage: move-assembler <COMMAND>

Commands:
  dis-script     Disassemble script bytecode
  dis-module     Disassemble module bytecode
  asm-script     Assemble script bytecode
  asm-module     Assemble module bytecode
  verify-script  Verify script bytecode
  verify-module  Verify module bytecode
  help           Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

# Disassemble a Move module bytecode file ('coin.mv') into human-readable form.
$ ./target/debug/move-assembler dis-module ./examples/coin.mv -o ./examples/coin.mv.asm

# After disassembling, you might edit the 'coin.mv.asm' file to modify the bytecode
# ...

# Reassemble the edited assembly file ('coin_modified.mv.asm') back into Move bytecode
$ ./target/debug/move-assembler asm-module ./examples/coin_modified.mv.asm -o ./examples/coin_modified.mv 

# Verify the reassembled Move module bytecode ('coin_modified.mv') to ensure its correctness
$ ./target/debug/move-assembler verify-module ./examples/coin_modified.mv
```
