//! Function
//! https://llvm.org/docs/BitCodeFormat.html#module-code-function-record

use crate::bitcode::blocks::param_attr::ParamAttrBlock;
use crate::bitcode::blocks::param_attr_group::ParamAttrList;
use crate::bitcode::blocks::typeblock::{TypeBlock, TypeBlockData, TypeBlockFunction};
use crate::bitcode::parse::StringRef;
use crate::{WError, WResult};

use crate::bitcode::{
    blocks::globalvar::{
        GlobalVarDllStorageClass, GlobalVarLinkAge, GlobalVarPreEmptionSpecifier,
        GlobalVarUnnamedAddr, GlobalVarVisibility,
    },
    entry::DataNext,
};

enum_parse! {
    pub(in crate::bitcode) FunctionCallingConv, u64;
    Debug, PartialEq,;
    Ccc = 0,
    Fastcc = 8,
    Coldcc = 9,
    Anyregcc = 13,
    PreserveMostcc = 14,
    PreserveAllcc = 15,
    Swiftcc  = 16,
    CxxFastTlscc = 17,
    Tailcc  = 18,
    CfguardCheckcc  = 19,
    Swifttailcc  = 20,
    X86Stdcallcc = 64,
    X86Fastcallcc = 65,
    ArmApcscc = 66,
    ArmAapcscc = 67,
    ArmAapcsVfpcc = 68,
}

#[derive(Debug, PartialEq)]
pub(in crate::bitcode) struct Function {
    pub(in crate::bitcode) name: String,
    pub(in crate::bitcode) ty: TypeBlockFunction,
    pub(in crate::bitcode) calling_conv: FunctionCallingConv,
    pub(in crate::bitcode) is_proto: bool,
    pub(in crate::bitcode) link_age: GlobalVarLinkAge,
    pub(in crate::bitcode) param_attr: Vec<ParamAttrList>,
    pub(in crate::bitcode) alignment: u8,
    pub(in crate::bitcode) section: usize,
    pub(in crate::bitcode) visibility: GlobalVarVisibility,
    pub(in crate::bitcode) gc: usize,
    pub(in crate::bitcode) unnamed_addr: GlobalVarUnnamedAddr,
    pub(in crate::bitcode) prologue_data: usize,
    pub(in crate::bitcode) dll_storage_class: GlobalVarDllStorageClass,
    pub(in crate::bitcode) comdat: usize,
    pub(in crate::bitcode) prefix_data: usize,
    pub(in crate::bitcode) personality_fn: usize,
    pub(in crate::bitcode) pre_emption_specifier: GlobalVarPreEmptionSpecifier,
}

pub(super) fn parse_function_record(
    data: Vec<u64>,
    strtab: &StringRef,
    data_types: &TypeBlock,
    param_attr: &ParamAttrBlock,
) -> WResult<Function> {
    let mut data = DataNext::from(data);

    let strtab_offset = data.next_usize("strtab_offset")?;
    let strtab_size = data.next_usize("strtab_size")?;
    let name = strtab
        .get(strtab_offset..(strtab_offset + strtab_size))
        .map(|v| v.iter().map(|v| *v as char).collect::<String>())
        .ok_or(WError::DataNotFound("parse function record name"))?;

    let ty = {
        let idx = data.next_usize("type")?;
        let ty = data_types.get(idx).cloned().ok_or(WError::DataNotAllowed(
            "parse function record ty",
            format!("{}/{}", idx, data_types.len()),
        ))?;
        let func = match ty.as_ref() {
            TypeBlockData::Function(v) => v,
            TypeBlockData::FunctionOld(v) => &v.func,
            v => {
                return Err(WError::DataNotAllowed(
                    "parse function record ty",
                    format!("{:?}", v),
                ));
            }
        };
        func.clone()
    };
    let calling_conv = data.next_convert::<FunctionCallingConv>("calling_conv")?;
    let is_proto = data.next_bool("is_proto")?;
    let link_age = data.next_convert::<GlobalVarLinkAge>("link_age")?;
    let param_attr = {
        let idx = data.next_usize("param_attr")?;
        if idx == 0 {
            return Err(WError::DataNotFound("parse function param_attr"));
        }
        param_attr
            .get(idx - 1)
            .cloned()
            .ok_or(WError::CodeNotAllowed(
                "parse function param_attr",
                idx as u32,
            ))?
    };
    let alignment = data.next("alignment")? as u8;
    let section = data.next_usize("section")?;
    let visibility = data.next_convert::<GlobalVarVisibility>("visibility")?;
    let gc = data.next_usize("gc")?;
    let unnamed_addr = data.next_convert::<GlobalVarUnnamedAddr>("unnamed_addr")?;
    let prologue_data = data.next_usize("prologue_data")?;
    let dll_storage_class = data.next_convert::<GlobalVarDllStorageClass>("dll_storage_class")?;
    let comdat = data.next_usize("comdat")?;
    let prefix_data = data.next_usize("prefix_data")?;
    let personality_fn = data.next_usize("personality_fn")?;
    let pre_emption_specifier =
        data.next_convert::<GlobalVarPreEmptionSpecifier>("pre_emption_specifier")?;

    // eprintln!("Function record: {}", name);

    Ok(Function {
        name,
        ty,
        calling_conv,
        is_proto,
        link_age,
        param_attr,
        alignment,
        section,
        visibility,
        gc,
        unnamed_addr,
        prologue_data,
        dll_storage_class,
        comdat,
        prefix_data,
        personality_fn,
        pre_emption_specifier,
    })
}
