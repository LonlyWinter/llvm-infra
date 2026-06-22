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
    DllImport = 5,
    DllExport = 6,
    ExternWeak = 7,
    Common = 8,
    Private = 9,
    WeakOdr = 10,
    LinkonceOdr = 11,
    AvailableExternally = 12,
}

impl GlobalVarLinkAge {
    pub(in crate::bitcode) fn is_internal(&self) -> bool {
        self.eq(&Self::Internal)
    }

    pub(in crate::bitcode) fn is_private(&self) -> bool {
        self.eq(&Self::Private)
    }

    pub(in crate::bitcode) fn is_local(&self) -> bool {
        self.is_internal() || self.is_private()
    }
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
    None = 0,
    Global = 1,
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
    // pub(in crate::bitcode) address_space: u16,
    pub(in crate::bitcode) is_const: bool,
    pub(in crate::bitcode) init_id: usize,
    pub(in crate::bitcode) link_age: GlobalVarLinkAge,
    pub(in crate::bitcode) align: Option<u16>,
    pub(in crate::bitcode) section: usize,
    pub(in crate::bitcode) visibility: GlobalVarVisibility,
    pub(in crate::bitcode) thread_local: GlobalVarThreadLocal,
    pub(in crate::bitcode) unnamed_addr: GlobalVarUnnamedAddr,
    pub(in crate::bitcode) externally_initialized: bool,
    pub(in crate::bitcode) dll_storage_class: GlobalVarDllStorageClass,
    pub(in crate::bitcode) comdat: Option<usize>,
    pub(in crate::bitcode) attributes: Option<usize>,
    pub(in crate::bitcode) pre_emption_specifier: Option<GlobalVarPreEmptionSpecifier>,
}


impl DataNext<u64> {
    pub(super) fn next_visibility(&mut self, link_age: &GlobalVarLinkAge, tag: &'static str) -> WResult<GlobalVarVisibility> {
        let r = self.next_convert::<GlobalVarVisibility>(tag);
        if !link_age.is_local() {
            r
        } else {
            Ok(GlobalVarVisibility::Default)
        }
    }

    pub(super) fn next_unname_addr(&mut self) -> GlobalVarUnnamedAddr {
        self.next_convert::<GlobalVarUnnamedAddr>("unnamed_addr")
            .unwrap_or(GlobalVarUnnamedAddr::None)
    }

    pub(super) fn next_dll_storage_class(&mut self, link_age: &GlobalVarLinkAge, tag: &'static str) -> WResult<GlobalVarDllStorageClass> {
        let r = if self.len() != 0 {
            let r = self.next_convert::<GlobalVarDllStorageClass>(tag)?;
            if !link_age.is_local() {
                r
            } else {
                GlobalVarDllStorageClass::Default
            }
        } else if !link_age.is_local() {
            match link_age {
                GlobalVarLinkAge::DllImport => GlobalVarDllStorageClass::DllImport,
                GlobalVarLinkAge::DllExport => GlobalVarDllStorageClass::DllExport,
                _ => GlobalVarDllStorageClass::Default,
            }
        } else {
            GlobalVarDllStorageClass::Default
        };
        Ok(r)
    }
}

pub(super) fn parse_global_var_record(
    data: Vec<u64>,
    strtab: &StringRef,
    data_types: &TypeBlock,
) -> WResult<GlobalVar> {
    let mut data = DataNext::from(data);
    let name = data.next_name(strtab)?;

    let ty = {
        let idx = data.next_usize("type")?;
        let ty = data_types.get(idx).cloned().ok_or(WError::DataNotAllowed(
            "global val ty",
            format!("{}/{}", idx, data_types.len()),
        ))?;
        if matches!(
            ty.as_ref(),
            TypeBlockData::Function(_) | TypeBlockData::FunctionOld(_)
        ) {
            return Err(WError::DataNotAllowed("global val ty", format!("{:?}", ty)));
        }
        ty
    };
    let info = data.next("global var info")?;
    let is_const = (info & 1) != 0;
    // let address_space = if (info & 2) != 0 {
    //     (info >> 2) as u16
    // } else {
    //     
    // };
    let init_id = data.next_usize("init_id")?;
    let link_age = data.next_convert::<GlobalVarLinkAge>("link_age")?;
    let align = data.next_alignment("global_var align").ok();
    let section = data.next_usize("section")?;
    let visibility = data.next_visibility(
        &link_age,
        "visibility in global var"
    ).unwrap_or(GlobalVarVisibility::Default);
    let thread_local = data.next_convert::<GlobalVarThreadLocal>("thread_local")
        .unwrap_or(GlobalVarThreadLocal::NotThreadLocal);
    let unnamed_addr = data.next_unname_addr();
    let externally_initialized = data.next_bool("externally_initialized")
        .unwrap_or(false);
    let dll_storage_class = data.next_dll_storage_class(
        &link_age,
        "dll_storage_class in global var"
    )?;
    let comdat = data.next_usize("comdat").ok();
    let attributes = data.next_usize("attributes").ok();
    let pre_emption_specifier = data.next_convert::<GlobalVarPreEmptionSpecifier>("pre_emption_specifier").ok();
    Ok(GlobalVar {
        name,
        ty,
        is_const,
        init_id,
        link_age,
        align,
        section,
        visibility,
        thread_local,
        unnamed_addr,
        externally_initialized,
        dll_storage_class,
        comdat,
        attributes,
        pre_emption_specifier,
    })
}
