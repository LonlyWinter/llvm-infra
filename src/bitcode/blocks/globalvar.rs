//! GlobalVar parse
//! https://llvm.org/docs/BitCodeFormat.html#module-code-globalvar-record

use crate::bitcode::blocks::typeblock::{TypeBlock, TypeBlockData, TypeBlockUnit};
use crate::bitcode::parse::StringRef;
use crate::{WError, WResult};

use crate::bitcode::entry::DataNext;

enum_parse! {
    pub(in crate::bitcode) GlobalVarLinkAge, u64;
    Debug, PartialEq,;
    External = 0,
    Weak = 1,
    Appending = 2,
    Internal = 3,
    Linkonce = 4,
    Dllimport = 5,
    Dllexport = 6,
    ExternWeak = 7,
    Common = 8,
    Private = 9,
    WeakOdr = 10,
    LinkonceOdr = 11,
    AvailableExternally = 12,
}

enum_parse! {
    pub(in crate::bitcode) GlobalVarVisibility, u64;
    Debug, PartialEq,;
    Default = 0,
    Hidden = 1,
    Protected = 2,
}

enum_parse! {
    pub(in crate::bitcode) GlobalVarThreadLocal, u64;
    Debug, PartialEq,;
    NotThreadLocal = 0,
    ThreadLocal = 1,
    LocalDynamic = 2,
    InitialExec = 3,
    LocalExec = 4,
}

enum_parse! {
    pub(in crate::bitcode) GlobalVarUnnamedAddr, u64;
    Debug, PartialEq,;
    Not = 0,
    Is = 1,
    Local = 2,
}

enum_parse! {
    pub(in crate::bitcode) GlobalVarDllStorageClass, u64;
    Debug, PartialEq,;
    Default = 0,
    DllImport = 1,
    DllExport = 2,
}

enum_parse! {
    pub(in crate::bitcode) GlobalVarPreEmptionSpecifier, u64;
    Debug, PartialEq,;
    DsoPreEmptable = 0,
    DsoLocal = 1,
}

#[derive(Debug, PartialEq)]
pub(in crate::bitcode) struct GlobalVar {
    pub(in crate::bitcode) name: String,
    pub(in crate::bitcode) ty: TypeBlockUnit,
    pub(in crate::bitcode) is_const: bool,
    pub(in crate::bitcode) init_id: usize,
    pub(in crate::bitcode) link_age: GlobalVarLinkAge,
    pub(in crate::bitcode) alignment: u8,
    pub(in crate::bitcode) section: usize,
    pub(in crate::bitcode) visibility: GlobalVarVisibility,
    pub(in crate::bitcode) thread_local: GlobalVarThreadLocal,
    pub(in crate::bitcode) unnamed_addr: GlobalVarUnnamedAddr,
    pub(in crate::bitcode) dll_storage_class: GlobalVarDllStorageClass,
    pub(in crate::bitcode) comdat: usize,
    pub(in crate::bitcode) attributes: usize,
    pub(in crate::bitcode) pre_emption_specifier: GlobalVarPreEmptionSpecifier,
}

pub(super) fn parse_global_var_record(data: Vec<u64>, strtab: &StringRef, data_types: &TypeBlock) -> WResult<GlobalVar> {
    let mut data = DataNext::from(data);

    let strtab_offset = data.next_usize("strtab_offset")?;
    let strtab_size = data.next_usize("strtab_size")?;
    let name = strtab
        .get(strtab_offset..(strtab_offset + strtab_size))
        .map(|v| v.iter().map(|v| *v as char).collect::<String>())
        .ok_or(WError::DataNotFound("parse global val name"))?;
    let ty = {
        let idx = data.next_usize("type")?;
        let ty = data_types.get(idx).cloned().ok_or(WError::DataNotAllowed(
            "parse global val ty",
            format!("{}/{}", idx, data_types.len()),
        ))?;
        if matches!(
            ty.as_ref(),
            TypeBlockData::Function(_) | TypeBlockData::FunctionOld(_)
        ) {
            return Err(WError::DataNotAllowed(
                "parse global val ty",
                format!("{:?}", ty),
            ));
        }
        ty
    };
    let is_const = data.next_bool("is_const")?;
    let init_id = data.next_usize("init_id")?;
    let link_age = data.next_convert::<GlobalVarLinkAge>("link_age")?;
    let alignment = data.next("alignment")? as u8;
    let section = data.next_usize("section")?;
    let visibility = data.next_convert::<GlobalVarVisibility>("visibility")?;
    let thread_local = data.next_convert::<GlobalVarThreadLocal>("thread_local")?;
    let unnamed_addr = data.next_convert::<GlobalVarUnnamedAddr>("unnamed_addr")?;
    let dll_storage_class = data.next_convert::<GlobalVarDllStorageClass>("dll_storage_class")?;
    let comdat = data.next_usize("comdat")?;
    let attributes = data.next_usize("attributes")?;
    let pre_emption_specifier =
        data.next_convert::<GlobalVarPreEmptionSpecifier>("pre_emption_specifier")?;

    // let mut ty = {
    //     let idx = data.next("type")? as usize;
    //     type_all
    //         .get(idx)
    //         .ok_or(WError::Parse(format!("No type found in parse global var: {}", idx)))?
    //         .clone()
    // };
    // let is_const = {
    //     let data = data.next("is_const")?;
    //     let explicit_type = (data & 2) != 0;
    //     let address_space = if explicit_type {
    //         (data >> 2) as usize
    //     } else {
    //         if let TypeBlockData::Pointer(p) = ty.as_ref() {
    //             p.space
    //         } else {
    //             return Err(WError::Parse(format!(
    //                 "No pointer type found in parse global var: {:?}",
    //                 ty
    //             )))
    //         }
    //     };
    //     (data & 1) != 0
    // };
    Ok(GlobalVar {
        name,
        ty,
        is_const,
        init_id,
        link_age,
        alignment,
        section,
        visibility,
        thread_local,
        unnamed_addr,
        dll_storage_class,
        comdat,
        attributes,
        pre_emption_specifier,
    })
}
