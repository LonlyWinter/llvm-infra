//! Add a function and a global var
//! 
//! ``` llvm
//! @chars_hex = private unnamed_addr constant [16 x i8] c"0123456789abcdef"
//! declare i32 @putchar(i32)
//! define void @print_chars(i32 %name_len, <128 x i8> %name) {
//! start:
//!   br label %print
//! print:
//!   %i_name = phi i32 [ 0, %start ], [ %i_next_name, %print ]
//!   %byte_name = extractelement <128 x i8> %name, i32 %i_name
//!   %byte_i32_name = zext i8 %byte_name to i32
//!   call i32 @putchar(i32 %byte_i32_name)
//!   %i_next_name = add i32 %i_name, 1
//!   %cond_name = icmp ult i32 %i_next_name, %name_len
//!   br i1 %cond_name, label %print, label %exit
//! exit:
//!   ret void
//! }
//! ```

use llvm_infra::{
    LLVM,
    WResult,
    meta::{
        Arg, Block, FunctionNative, FunctionProto, GlobalVar,
        InstrCalc, InstrCalcBinOP, InstrCalcBinOPOp, InstrCalcCall,
        InstrCalcCast, InstrCalcCastOp, InstrCalcCmp, InstrCalcCmpIntOp,
        InstrCalcCmpOp, InstrCalcExtractElt, InstrPhi, InstrTerm,
        Ty, TyBasic, ValBasic, Var
    },
    utils::{ID_STATE_FUNC, ID_STATE_GLOBAL_VAR, ID_TAG_BLOCK}
};

fn add_global_var(llvm: &mut LLVM) -> WResult<()> {
    // global var
    let var = Var::convert_str_to_array_i8("0123456789abcdef");
    let var = GlobalVar::generate(var, &mut llvm.id);
    let name_hex_chars = llvm.id.generate(
        ID_STATE_GLOBAL_VAR,
        "chars_hex",
    );
    llvm.vars_global.insert(name_hex_chars, var);
    Ok(())
}


fn add_func(llvm: &mut LLVM) -> WResult<()> {
    // func putchar
    let t: Ty = TyBasic::Int(32).into();
    let ret_putchar = Arg::from(t);
    let inps_putchar = vec![ret_putchar.clone(), ];
    let name_func_putchar = llvm.id.generate(
        ID_STATE_FUNC,
        "putchar",
    );
    let func_putchar = FunctionProto::generate(
        ret_putchar.clone(),
        inps_putchar.clone(),
        &mut llvm.id
    );

    // func print name
    let ret: Ty = TyBasic::Void.into();
    let ret_print_name = Arg::from(ret);
    let arg1: Ty = TyBasic::Int(32).into();
    let arg2 = Ty::Vector(Box::new(TyBasic::Int(8).into()), 128);
    let inps_print_name = vec![
        Arg::from(arg1),
        Arg::from(arg2)
    ];
    let (
        name_func_print_name,
        mut func_print_name
    ) = FunctionNative::generate(
        "print_chars",
        ret_print_name.clone(),
        inps_print_name.clone(),
        vec!["name_len", "name"],
        &mut llvm.id
    )?;
    // block start
    let name_func = name_func_print_name.to_string();
    let block_name_start = llvm.id.generate_with_name(
        ID_TAG_BLOCK,
        &name_func,
        "start"
    );
    let block_name_print = llvm.id.generate_with_name(
        ID_TAG_BLOCK,
        &name_func,
        "print",
    );
    let block_name_exit = llvm.id.generate_with_name(
        ID_TAG_BLOCK,
        &name_func,
        "exit",
    );
    let block_start = Block {
        name: block_name_start.clone(),
        instrs_phi: Vec::with_capacity(0),
        instrs_calc: Vec::with_capacity(0),
        instr_term: InstrTerm::DirectBr(block_name_print.clone()),
    };
    let block_exit = Block {
        name: block_name_exit.clone(),
        instrs_phi: Vec::with_capacity(0),
        instrs_calc: Vec::with_capacity(0),
        instr_term: InstrTerm::Ret(None),
    };
    
    let i_name = func_print_name.generate_var(
        &name_func,
        "i_name",
        TyBasic::Int(32).into(),
        &mut llvm.id
    );
    let byte_name = func_print_name.generate_var(
        &name_func,
        "byte_name",
        TyBasic::Int(8).into(),
        &mut llvm.id
    );
    let byte_i32_name = func_print_name.generate_var(
        &name_func,
        "byte_i32_name",
        TyBasic::Int(32).into(),
        &mut llvm.id
    );
    let i_next_name = func_print_name.generate_var(
        &name_func,
        "i_next_name",
        TyBasic::Int(32).into(),
        &mut llvm.id
    );
    let cond_name = func_print_name.generate_var(
        &name_func,
        "cond_name",
        TyBasic::Int(1).into(),
        &mut llvm.id
    );
    let const_0 = func_print_name.generate_const(
        Var::Basic(TyBasic::Int(32), Some(ValBasic::I32(0))),
        &mut llvm.id
    );
    let const_1 = func_print_name.generate_const(
        Var::Basic(TyBasic::Int(32), Some(ValBasic::I32(1))),
        &mut llvm.id
    );
    let instr_call = InstrCalcCall::generate(
        None,
        vec![byte_i32_name.clone(), ],
        name_func_putchar.clone(),
        &func_putchar,
        &mut llvm.id
    )?;
    let block_print_name = Block {
        name: block_name_print.clone(),
        instrs_phi: vec![InstrPhi {
            res: i_name.clone(),
            srcs: vec![
                (
                    block_name_start.clone(),
                    const_0
                ),
                (
                    block_name_print.clone(),
                    i_next_name.clone()
                ),
            ]
        },],
        instrs_calc: vec![
            InstrCalc::ExtractElt(InstrCalcExtractElt {
                res: byte_name.clone(),
                src: func_print_name.inps[1].clone(),
                idx: i_name.clone(),
            }).into(),
            InstrCalc::Cast(InstrCalcCast {
                res: byte_i32_name,
                op: InstrCalcCastOp::ZExt(false),
                val: byte_name,
            }).into(),
            InstrCalc::Call(instr_call).into(),
            InstrCalc::BinOp(InstrCalcBinOP {
                res: i_next_name.clone(),
                op: InstrCalcBinOPOp::Add(false, false),
                left: i_name,
                right: const_1,
            }).into(),
            InstrCalc::Cmp(InstrCalcCmp {
                res: cond_name.clone(),
                op: InstrCalcCmpOp::Int(
                    InstrCalcCmpIntOp::Ult,
                    true
                ),
                left: i_next_name,
                right: func_print_name.inps[0].clone()
            }).into(),
        ],
        instr_term: InstrTerm::CondBr(
            cond_name,
            block_name_print,
            block_name_exit,
        ),
    };
    func_print_name.blocks.push(block_start);
    func_print_name.blocks.push(block_print_name);
    func_print_name.blocks.push(block_exit);
    func_print_name.vars.shrink_to_fit();
    func_print_name.blocks.shrink_to_fit();

    llvm.funcs_native.insert(name_func_print_name, func_print_name);
    Ok(())
}


fn main() -> WResult<()> {
    let mut llvm = LLVM::from_file_bc("data/aes128.bc")?;
    llvm.save_file_ll("aes128.test.raw.ll")?;
    add_func(&mut llvm)?;
    add_global_var(&mut llvm)?;
    llvm.save_file_ll("aes128.test.new.ll")?;
    Ok(())
}