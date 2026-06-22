# llvm-infra

[![Latest version](https://img.shields.io/crates/v/llvm-infra.svg)](https://crates.io/crates/llvm-infra)
[![License](https://img.shields.io/badge/license-GPLv3-blue?style=flat)](https://github.com/LonlyWinter/llvm-infra/blob/master/COPYING)

LLVM Compiler Infrastructure in Rust

## LLVM object

### 1. Parse from `.bc` file

``` rust
let llvm = LLVM::from_file_bc("file.bc")?;
```

### 2. Save with `.ll` format

``` rust
llvm.save_file_ll("file.ll")?;
```

### 3. All instructions

``` rust
for (func_name, func_meta) in llvm.funcs_native {
    let vars = func_meta.vars;
    let ty_ret = func_meta.base.ret;
    let ty_args = func_meta.base.args;
    let name_inps = func_meta.inps;
    // single basic block
    for block in func_meta.blocks {
        let block_name = block.name;
        let instrs_phi = block.instrs_phi;
        let instr_term = block.instr_term;
        for instr_with_meta in block.instrs_calc {
            // single instruction
            match instr_with_meta.instr {
                InstrCalc::Alloca(instr_alloca) => {},
                InstrCalc::Call(instr_call) => {},
                InstrCalc::Gep(instr_gep) => {},
                ...
            }
        }
    }
}
```

### 4. Insert functions/instructions

example [`add_func_and_globalvar`](./examples/add_func_and_globalvar.rs)


## Todo

- LLVM passes

## License

GPLv3

## Author

winter, winter_lonely@foxmail.com
