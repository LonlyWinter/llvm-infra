//! Function
//! https://llvm.org/docs/BitCodeFormat.html#module-code-function-record

use std::rc::Rc;

use crate::bitcode::blocks::param_attr::ParamAttrBlock;
use crate::bitcode::blocks::param_attr_group::ParamAttrUnit;
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

    // The default llvm calling convention, compatible with C. This convention
    // is the only one that supports varargs calls. As with typical C calling
    // conventions, the callee/caller have to tolerate certain amounts of
    // prototype mismatch.
    C = 0,

    // Generic LLVM calling conventions. None of these support varargs calls,
    // and all assume that the caller and callee prototype exactly match.

    // Attempts to make calls as fast as possible (e.g. by passing things in
    // registers).
    Fast = 8,

    // Attempts to make code in the caller as efficient as possible under the
    // assumption that the call is not commonly executed. As such, these calls
    // often preserve all registers so that the call does not break any live
    // ranges in the caller side.
    Cold = 9,

    // Used by the Glasgow Haskell Compiler (GHC).
    Ghc = 10,

    // Used by the High-Performance Erlang Compiler (HiPE).
    HiPE = 11,

    // OBSOLETED - Used for stack based JavaScript calls
    WebKitJs = 12,

    // Used for dynamic register based calls (e.g. stackmap and patchpoint
    // intrinsics).
    AnyReg = 13,

    // Used for runtime calls that preserves most registers.
    PreserveMost = 14,

    // Used for runtime calls that preserves (almost) all registers.
    PreserveAll = 15,

    // Calling convention for Swift.
    Swift = 16,

    // Used for access functions.
    CxxFastTls = 17,

    // Attemps to make calls as fast as possible while guaranteeing that tail
    // call optimization can always be performed.
    Tail = 18,

    // Special calling convention on Windows for calling the Control Guard
    // Check ICall funtion. The function takes exactly one argument (address of
    // the target function) passed in the first argument register, and has no
    // return value. All register values are preserved.
    CFGuardCheck = 19,

    // This follows the Swift calling convention in how arguments are passed
    // but guarantees tail calls will be made by making the callee clean up
    // their stack.
    SwiftTail = 20,

    // Used for runtime calls that preserves none general registers.
    PreserveNone = 21,

    // stdcall is mostly used by the Win32 API. It is basically the same as the
    // C convention with the difference in that the callee is responsible for
    // popping the arguments from the stack.
    X86StdCall = 64,

    // 'fast' analog of X86_StdCall. Passes first two arguments in ECX:EDX
    // registers, others - via stack. Callee is responsible for stack cleaning.
    X86FastCall = 65,

    // ARM Procedure Calling Standard (obsolete, but still used on some
    // targets).
    ARMAPCS = 66,

    // ARM Architecture Procedure Calling Standard calling convention (aka
    // EABI). Soft float variant.
    ARMAAPCS = 67,

    // Same as ARM_AAPCS, but uses hard floating point ABI.
    ARMAAPCSVFP = 68,

    // Used for MSP430 interrupt routines.
    MSP430INTR = 69,

    // Similar to X86_StdCall. Passes first argument in ECX, others via stack.
    // Callee is responsible for stack cleaning. MSVC uses this by default for
    // methods in its ABI.
    X86ThisCall = 70,

    // Call to a PTX kernel. Passes all arguments in parameter space.
    PTXKernel = 71,

    // Call to a PTX device function. Passes all arguments in register or
    // parameter space.
    PTXDevice = 72,

    // Used for SPIR non-kernel device functions. No lowering or expansion of
    // arguments. Structures are passed as a pointer to a struct with the
    // byval attribute. Functions can only call SPIR_FUNC and SPIR_KERNEL
    // functions. Functions can only have zero or one return values. Variable
    // arguments are not allowed, except for printf. How arguments/return
    // values are lowered are not specified. Functions are only visible to the
    // devices.
    SPIRFUNC = 75,

    // Used for SPIR kernel functions. Inherits the restrictions of SPIR_FUNC,
    // except it cannot have non-void return values, it cannot have variable
    // arguments, it can also be called by the host or it is externally
    // visible.
    SPIRKERNEL = 76,

    // Used for Intel OpenCL built-ins.
    IntelOCLBI = 77,

    // The C convention as specified in the x86-64 supplement to the System V
    // ABI, used on most non-Windows systems.
    X8664SysV = 78,

    // The C convention as implemented on Windows/x86-64 and AArch64. It
    // differs from the more common \c X86_64_SysV convention in a number of
    // ways, most notably in that XMM registers used to pass arguments are
    // shadowed by GPRs, and vice versa. On AArch64, this is identical to the
    // normal C (AAPCS) calling convention for normal functions, but floats are
    // passed in integer registers to variadic functions.
    Win64 = 79,

    // MSVC calling convention that passes vectors and vector aggregates in SSE
    // registers.
    X86VectorCall = 80,

    // Placeholders for HHVM calling conventions (deprecated, removed).
    DUMMYHHVM = 81,
    DUMMYHHVMC = 82,

    // x86 hardware interrupt context. Callee may take one or two parameters,
    // where the 1st represents a pointer to hardware context frame and the 2nd
    // represents hardware error code, the presence of the later depends on the
    // interrupt vector taken. Valid for both 32- and 64-bit subtargets.
    X86INTR = 83,

    // Used for AVR interrupt routines.
    AVRINTR = 84,

    // Used for AVR signal routines.
    AVRSIGNAL = 85,

    // Used for special AVR rtlib functions which have an "optimized"
    // convention to preserve registers.
    AVRBUILTIN = 86,

    // Used for Mesa vertex shaders, or AMDPAL last shader stage before
    // rasterization (vertex shader if tessellation and geometry are not in
    // use, or otherwise copy shader if one is needed).
    AMDGPUVS = 87,

    // Used for Mesa/AMDPAL geometry shaders.
    AMDGPUGS = 88,

    // Used for Mesa/AMDPAL pixel shaders.
    AMDGPUPS = 89,

    // Used for Mesa/AMDPAL compute shaders.
    AMDGPUCS = 90,

    // Used for AMDGPU code object kernels.
    AMDGPUKERNEL = 91,

    // Register calling convention used for parameters transfer optimization
    X86RegCall = 92,

    // Used for Mesa/AMDPAL hull shaders (= tessellation control shaders).
    AMDGPUHS = 93,

    // Used for special MSP430 rtlib functions which have an "optimized"
    // convention using additional registers.
    MSP430BUILTIN = 94,

    // Used for AMDPAL vertex shader if tessellation is in use.
    AMDGPULS = 95,

    // Used for AMDPAL shader stage before geometry shader if geometry is in
    // use. So either the domain (= tessellation evaluation) shader if
    // tessellation is in use, or otherwise the vertex shader.
    AMDGPUES = 96,

    // Used between AArch64 Advanced SIMD functions
    AArch64VectorCall = 97,

    // Used between AArch64 SVE functions
    AArch64SVEVectorCall = 98,

    // For emscripten __invoke_* functions. The first argument is required to
    // be the function ptr being indirectly called. The remainder matches the
    // regular calling convention.
    WASMEmscriptenInvoke = 99,

    // Used for AMD graphics targets.
    AMDGPUGfx = 100,

    // Used for M68k interrupt routines.
    M68kINTR = 101,

    // Preserve X0-X13, X19-X29, SP, Z0-Z31, P0-P15.
    AArch64SMEABISupportRoutinesPreserveMostFromX0 = 102,

    // Preserve X2-X15, X19-X29, SP, Z0-Z31, P0-P15.
    AArch64SMEABISupportRoutinesPreserveMostFromX2 = 103,

    // Used on AMDGPUs to give the middle-end more control over argument
    // placement.
    AMDGPUCSChain = 104,

    // Used on AMDGPUs to give the middle-end more control over argument
    // placement. Preserves active lane values for input VGPRs.
    AMDGPUCSChainPreserve = 105,

    // Used for M68k rtd-based CC (similar to X86's stdcall).
    M68kRTD = 106,

    // Used by GraalVM. Two additional registers are reserved.
    GRAAL = 107,

    // Calling convention used in the ARM64EC ABI to implement calls between
    // x64 code and thunks. This is basically the x64 calling convention using
    // ARM64 register names. The first parameter is mapped to x9.
    ARM64ECThunkX64 = 108,

    // Calling convention used in the ARM64EC ABI to implement calls between
    // ARM64 code and thunks. This is just the ARM64 calling convention,
    // except that the first parameter is mapped to x9.
    ARM64ECThunkNative = 109,

    // Calling convention used for RISC-V V-extension.
    RISCVVectorCall = 110,

    // Preserve X1-X15, X19-X29, SP, Z0-Z31, P0-P15.
    AArch64SMEABISupportRoutinesPreserveMostFromX1 = 111,

    // Calling convention used for RISC-V V-extension fixed vectors.
    RISCVVLSCall32 = 112,
    RISCVVLSCall64 = 113,
    RISCVVLSCall128 = 114,
    RISCVVLSCall256 = 115,
    RISCVVLSCall512 = 116,
    RISCVVLSCall1024 = 117,
    RISCVVLSCall2048 = 118,
    RISCVVLSCall4096 = 119,
    RISCVVLSCall8192 = 120,
    RISCVVLSCall16384 = 121,
    RISCVVLSCall32768 = 122,
    RISCVVLSCall65536 = 123,

    // Calling convention for AMDGPU whole wave functions.
    AMDGPUGfxWholeWave = 124,

    // Calling convention used for CHERIoT when crossing a protection boundary.
    CHERIoTCompartmentCall = 125,
    // Calling convention used for the callee of CHERIoT_CompartmentCall.
    // Ignores the first two capability arguments and the first integer
    // argument, zeroes all unused return registers on return.
    CHERIoTCompartmentCallee = 126,
    // Calling convention used for CHERIoT for cross-library calls to a
    // stateless compartment.
    CHERIoTLibraryCall = 127,
}

#[derive(Debug, PartialEq)]
pub(in crate::bitcode) struct Function {
    pub(in crate::bitcode) name: Rc<String>,
    pub(in crate::bitcode) ty: Rc<TypeBlockFunction>,
    pub(in crate::bitcode) calling_conv: FunctionCallingConv,
    pub(in crate::bitcode) is_proto: bool,
    pub(in crate::bitcode) link_age: GlobalVarLinkAge,
    pub(in crate::bitcode) param_attr: ParamAttrUnit,
    pub(in crate::bitcode) align: u16,
    /// module block section name
    pub(in crate::bitcode) section: Option<usize>,
    pub(in crate::bitcode) visibility: GlobalVarVisibility,
    pub(in crate::bitcode) gc: Option<usize>,
    pub(in crate::bitcode) unnamed_addr: GlobalVarUnnamedAddr,
    pub(in crate::bitcode) prologue_data: usize,
    pub(in crate::bitcode) dll_storage_class: GlobalVarDllStorageClass,
    pub(in crate::bitcode) comdat: usize,
    pub(in crate::bitcode) prefix_data: usize,
    pub(in crate::bitcode) personality_fn: usize,
    pub(in crate::bitcode) pre_emption_specifier: GlobalVarPreEmptionSpecifier,
    pub(in crate::bitcode) partition: String,
}

pub(super) fn parse_function_record(
    data: Vec<u64>,
    strtab: &StringRef,
    data_types: &TypeBlock,
    param_attr: &ParamAttrBlock,
) -> WResult<Function> {
    let mut data = DataNext::from(data);

    let name = data.next_name(strtab)?;

    let ty = {
        let idx = data.next_usize("type")?;
        let ty = data_types.get(idx).cloned().ok_or(WError::DataNotAllowed(
            "function record ty",
            format!("{}/{}", idx, data_types.len()),
        ))?;
        let func = match ty.as_ref() {
            TypeBlockData::Function(v) => v,
            TypeBlockData::FunctionOld(v) => &v.func,
            v => {
                return Err(WError::DataNotAllowed(
                    "function record ty",
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
            return Err(WError::DataNotFound("function param_attr"));
        }
        param_attr
            .get(idx - 1)
            .cloned()
            .ok_or(WError::CodeNotAllowed("function param_attr", idx as u32))?
    };
    let align = data.next_alignment("function align").unwrap_or(0);
    let section = data.next_usize("section").ok();
    let visibility = data.next_visibility(
        &link_age,
        "visibility in function record"
    )?;
    let gc = data.next_usize("gc").ok();
    let unnamed_addr = data.next_unname_addr();
    let prologue_data = data.next_usize("prologue_data")?;
    let dll_storage_class = data.next_dll_storage_class(
        &link_age,
        "dll_storage_class in function record"
    )?;
    let comdat = data.next_usize("comdat")?;
    let prefix_data = data.next_usize("prefix_data")?;
    let personality_fn = data.next_usize("personality_fn")?;
    let pre_emption_specifier = data
        .next_convert::<GlobalVarPreEmptionSpecifier>("pre_emption_specifier")?;
    let partition = data.next_name(strtab).unwrap_or_default();

    // eprintln!("Function record: {}", name);

    Ok(Function {
        name: Rc::new(name),
        ty,
        calling_conv,
        is_proto,
        link_age,
        param_attr,
        align,
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
        partition
    })
}
