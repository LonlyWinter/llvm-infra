# llvm-infra

[![Latest version](https://img.shields.io/crates/v/llvm-infra.svg)](https://crates.io/crates/llvm-infra)
[![License](https://img.shields.io/badge/license-GPLv3-blue?style=flat)](https://github.com/LonlyWinter/llvm-infra/blob/master/COPYING)

LLVM Compiler Infrastructure in Rust

## LLVM from `.bc` file

``` rust
// parse from .bc file
let llvm = LLVM::from_file_bc("file.bc")?;

// display
println!("{}", llvm);

// all native function
for (func_name, func_meta) in llvm.funcs_native {
    // function related vars: GlobalVar/Var/Const/Metadata
    let vars = func_meta.vars;

    let ty_ret = func_meta.base.ret;
    let ty_inps = func_meta.base.inps;
    let name_inps = func_meta.inps;
    // single basic block
    for block in func_meta.blocks {
        let block_name = block.name;
        let instrs_phi = block.instrs_phi;
        let instr_term = block.instr_term;
        for instr in block.instrs_calc {
            // single instruction
            match instr {
                InstrCalc::Alloca(instr_alloca) => {},
                InstrCalc::Call(instr_call) => {},
                InstrCalc::Gep(instr_gep) => {},
                ...
            }
        }
    }
}
```

## Todo

- LLVM passes

## Author

winter, winter_lonely@foxmail.com
