//! Attr

use std::fmt::Display;

use crate::{meta::{ID, IDGenerater}, utils::ID_STATE_ATTR};

pub trait AttrID {
    fn to_str(&self) -> &'static str;
}

impl IDGenerater {
    pub fn generate_attr<T: AttrID>(&mut self, attr: T) -> ID {
        self.generate(
            ID_STATE_ATTR,
            attr.to_str()
        )
    }
}

impl ID {
    /// 匹配attr
    pub fn is_attr<T: AttrID>(&self, attr: &T) -> bool {
        self.with_data(| v | v == attr.to_str())
            .unwrap_or(false)
    }
    
    /// 匹配attr
    pub fn is_any_attr<T: AttrID>(&self, attrs: &[T]) -> bool {
        attrs.iter().any(| v | self.is_attr(v))
    }
}


macro_rules! enum_attr {
    (
        $name:ident;
        // no related data
        $($id1:ident = $val1:expr)*;
        // on data
        $($id2:ident = $data2:ident = $val2:expr)*
    ) => {
        pub enum $name {
            $(
                $id1,
            )*
            $(
                $id2($data2),
            )*
        }
        
        impl AttrID for $name {
            fn to_str(&self) -> &'static str {
                match self {
                    $(
                        Self::$id1 => concat!(stringify!($name), "_", $val1),
                    )*
                    $(
                        Self::$id2(_) => $val2,
                    )*
                }
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$id1 => write!(f, $val1),
                    )*
                    $(
                        Self::$id2(_) => write!(f, $val2),
                    )*
                }
            }
        }
    };
}

pub const ATTR_LINK_AGE_EXTERNAL: &str = "";
pub const ATTR_LINK_AGE_AVAILABLEEXTERNALLY: &str = "available_externally";
pub const ATTR_LINK_AGE_LINKONCEODR: &str = "linkonce_odr";
pub const ATTR_LINK_AGE_EXTERNWEAK: &str = "extern_weak";
pub const ATTR_LINK_AGE_WEAKODR: &str = "weak_odr";
pub const ATTR_LINK_AGE_WEAK: &str = "weak";
pub const ATTR_LINK_AGE_APPENDING: &str = "appending";
pub const ATTR_LINK_AGE_INTERNAL: &str = "internal";
pub const ATTR_LINK_AGE_LINKONCE: &str = "linkonce";
pub const ATTR_LINK_AGE_DLLIMPORT: &str = "dllimport";
pub const ATTR_LINK_AGE_DLLEXPORT: &str = "dllexport";
pub const ATTR_LINK_AGE_COMMON: &str = "common";
pub const ATTR_LINK_AGE_PRIVATE: &str = "private";

pub const ATTR_VISIBILITY_DEFAULT: &str = "";
pub const ATTR_VISIBILITY_HIDDEN: &str = "hidden";
pub const ATTR_VISIBILITY_PROTECTED: &str = "protected";

pub const ATTR_DLL_STORAGE_CLASS_DEFAULT: &str = "";
pub const ATTR_DLL_STORAGE_CLASS_DLLIMPORT: &str = "dllimport";
pub const ATTR_DLL_STORAGE_CLASS_DLLEXPORT: &str = "dllexport";

pub const ATTR_THREAD_LOCAL_NOTTHREADLOCAL: &str = "";
pub const ATTR_THREAD_LOCAL_THREADLOCAL: &str = "thread_local";
pub const ATTR_THREAD_LOCAL_LOCALDYNAMIC: &str = "thread_local(localdynamic)";
pub const ATTR_THREAD_LOCAL_INITIALEXEC: &str = "thread_local(initialexec)";
pub const ATTR_THREAD_LOCAL_LOCALEXEC: &str = "thread_local(locexec)";

pub const ATTR_UNNAMED_ADDR_NONE: &str = "";
pub const ATTR_UNNAMED_ADDR_GLOBAL: &str = "unnamed_addr";
pub const ATTR_UNNAMED_ADDR_LOCAL: &str = "local_unnamed_addr";


pub const ATTR_CALLING_CONV_CCC: &str = "";
pub const ATTR_CALLING_CONV_FASTCC: &str = "fastcc";
pub const ATTR_CALLING_CONV_COLDCC: &str = "coldcc";
pub const ATTR_CALLING_CONV_GHCCC: &str = "ghccc";
pub const ATTR_CALLING_CONV_WEBKITJSCC: &str = "webkit_jscc";
pub const ATTR_CALLING_CONV_ANYREGCC: &str = "anyregcc";
pub const ATTR_CALLING_CONV_PRESERVEMOSTCC: &str = "preserve_mostcc";
pub const ATTR_CALLING_CONV_PRESERVEALLCC: &str = "preserve_allcc";
pub const ATTR_CALLING_CONV_SWIFTCC: &str = "swiftcc";
pub const ATTR_CALLING_CONV_CXXFASTTLSCC: &str = "cxx_fast_tlscc";
pub const ATTR_CALLING_CONV_TAILCC: &str = "tailcc";
pub const ATTR_CALLING_CONV_CFGUARDCHECKCC: &str = "cfguard_checkcc";
pub const ATTR_CALLING_CONV_SWIFTTAILCC: &str = "swifttailcc";
pub const ATTR_CALLING_CONV_X86STDCALLCC: &str = "x86_stdcallcc";
pub const ATTR_CALLING_CONV_X86FASTCALLCC: &str = "x86_fastcallcc";
pub const ATTR_CALLING_CONV_ARMAPCSCC: &str = "arm_apcscc";
pub const ATTR_CALLING_CONV_ARMAAPCSCC: &str = "arm_aapcscc";
pub const ATTR_CALLING_CONV_ARMAAPCSVFPCC: &str = "arm_aapcs_vfpcc";


enum_attr! { InstrCallTailTy;
    None = ""
    Tail = "tail"
    Must = "musttail"
    No = "notail"
    ;
}

enum_attr! { FunctionCallingConv;
    C = ""
    Fast = "fastcc"
    Cold = "coldcc"
    WebKitJs = "webkit_jscc"
    AnyReg = "anyregcc"
    PreserveMost = "preserve_mostcc"
    PreserveAll = "preserve_allcc"
    PreserveNone = "preserve_nonecc"
    CxxFastTls = "cxx_fast_tlscc"
    Ghc = "ghccc"
    Tail = "tailcc"
    GRAAL = "graalcc"
    CFGuardCheck = "cfguard_checkcc"
    X86StdCall = "x86_stdcallcc"
    X86FastCall = "x86_fastcallcc"
    X86ThisCall = "x86_thiscallcc"
    X86RegCall = "x86_regcallcc"
    X86VectorCall = "x86_vectorcallcc"
    IntelOCLBI = "intel_ocl_bicc"
    ARMAPCS = "arm_apcscc"
    ARMAAPCS = "arm_aapcscc"
    ARMAAPCSVFP = "arm_aapcs_vfpcc"
    AArch64VectorCall = "aarch64_vector_pcs"
    AArch64SVEVectorCall = "aarch64_sve_vector_pcs"
    AArch64SMEABISupportRoutinesPreserveMostFromX0 = "aarch64_sme_preservemost_from_x0"
    AArch64SMEABISupportRoutinesPreserveMostFromX1 = "aarch64_sme_preservemost_from_x1"
    AArch64SMEABISupportRoutinesPreserveMostFromX2 = "aarch64_sme_preservemost_from_x2"
    MSP430INTR = "msp430_intrcc"
    AVRINTR = "avr_intrcc "
    AVRSIGNAL = "avr_signalcc "
    PTXKernel = "ptx_kernel"
    PTXDevice = "ptx_device"
    X8664SysV = "x86_64_sysvcc"
    Win64 = "win64cc"
    SPIRFUNC = "spir_func"
    SPIRKERNEL = "spir_kernel"
    Swift = "swiftcc"
    SwiftTail = "swifttailcc"
    X86INTR = "x86_intrcc"
    DUMMYHHVM = "hhvmcc"
    DUMMYHHVMC = "hhvm_ccc"
    AMDGPUVS = "amdgpu_vs"
    AMDGPULS = "amdgpu_ls"
    AMDGPUHS = "amdgpu_hs"
    AMDGPUES = "amdgpu_es"
    AMDGPUGS = "amdgpu_gs"
    AMDGPUPS = "amdgpu_ps"
    AMDGPUCS = "amdgpu_cs"
    AMDGPUCSChain = "amdgpu_cs_chain"
    AMDGPUCSChainPreserve = "amdgpu_cs_chain_preserve"
    AMDGPUKERNEL = "amdgpu_kernel"
    AMDGPUGfx = "amdgpu_gfx"
    AMDGPUGfxWholeWave = "amdgpu_gfx_whole_wave"
    M68kRTD = "m68k_rtdcc"
    RISCVVectorCall = "riscv_vector_cc"
    RISCVVLSCall32 = "riscv_vls_cc(32)"
    RISCVVLSCall64 = "riscv_vls_cc(64)"
    RISCVVLSCall128 = "riscv_vls_cc(128)"
    RISCVVLSCall256 = "riscv_vls_cc(256)"
    RISCVVLSCall512 = "riscv_vls_cc(512)"
    RISCVVLSCall1024 = "riscv_vls_cc(1024)"
    RISCVVLSCall2048 = "riscv_vls_cc(2048)"
    RISCVVLSCall4096 = "riscv_vls_cc(4096)"
    RISCVVLSCall8192 = "riscv_vls_cc(8192)"
    RISCVVLSCall16384 = "riscv_vls_cc(16384)"
    RISCVVLSCall32768 = "riscv_vls_cc(32768)"
    RISCVVLSCall65536 = "riscv_vls_cc(65536)"
    CHERIoTCompartmentCall = "cheriot_compartmentcallcc"
    CHERIoTCompartmentCallee = "cheriot_compartmentcalleecc"
    CHERIoTLibraryCall = "cheriot_librarycallcc"

    HiPE = "cc 11"
    AVRBUILTIN = "cc 35"
    MSP430BUILTIN = "cc 43"
    WASMEmscriptenInvoke = "cc 48"
    M68kINTR = "cc 50"
    ARM64ECThunkX64 = "cc 57"
    ARM64ECThunkNative = "cc 58"
    ;
}

enum_attr! { InstrCalcAtomicOrdering;
    NotAtomic = ""
    Unordered = "unordered"
    Monotonic = "monotonic"
    Acquire = "acquire"
    Release = "release"
    AcquireRelease = "acq_rel"
    SequentiallyConsistent = "seq_cst"
    ;
}


enum_attr! { InstrCalcAtomicRMWOp;
    Xchg = "xchg"
    Add = "add"
    Sub = "sub"
    And = "and"
    Nand = "nand"
    Or = "or"
    Xor = "xor"
    Max = "max"
    Min = "min"
    Umax = "umax"
    Umin = "umin"
    Fadd = "fadd"
    Fsub = "fsub"
    Fmax = "fmax"
    Fmin = "fmin"
    UincWrap = "uincwrap"
    UdecWrap = "udecwrap"
    UsubCond = "usubcond"
    UsubSat = "usubsat"
    Fmaximum = "fmaximum"
    Fminimum = "fminimum"
    ;
}

enum_attr! { InstrCalcLandingPadClauseOp;
    Catch = "catch"
    Filter = "filter"
    ;
}

enum_attr! { InstrCalcUnOpOp;
    ;
    FNeg = bool = "fneg"
}

enum_attr! { InstrFuncAsmDialect;
    Intel = "inteldialect"
    Att = ""
    ;
}
enum_attr! { GlobalVarLinkAge;
    External = ""
    AvailableExternally = "available_externally"
    LinkonceOdr = "linkonce_odr"
    ExternWeak = "extern_weak"
    WeakOdr = "weak_odr"
    Weak = "weak"
    Appending = "appending"
    Internal = "internal"
    Linkonce = "linkonce"
    DllImport = "dllimport"
    DllExport = "dllexport"
    Common = "common"
    Private = "private"
    ;
}

enum_attr! { GlobalVarVisibility;
    Default = ""
    Hidden = "hidden"
    Protected = "protected"
    ;
}

enum_attr! { GlobalVarUnnamedAddr;
    None = ""
    Global = "unnamed_addr"
    Local = "local_unnamed_addr"
    ;
}

enum_attr! { GlobalVarThreadLocal;
    NotThreadLocal = ""
    ThreadLocal = "thread_local"
    LocalDynamic = "thread_local(localdynamic)"
    InitialExec = "thread_local(initialexec)"
    LocalExec = "thread_local(locexec)"
    ;
}

enum_attr! { GlobalVarDllStorageClass;
    Default = ""
    DllImport = "dllimport"
    DllExport = "dllexport"
    ;
}


pub const ATTR_PARAM_ALLOCALIGN: &str = "allocalign";
pub const ATTR_PARAM_ALLOCATEDPOINTER: &str = "allocatedpointer";
pub const ATTR_PARAM_ALWAYSINLINE: &str = "alwaysinline";
pub const ATTR_PARAM_BUILTIN: &str = "builtin";
pub const ATTR_PARAM_COLD: &str = "cold";
pub const ATTR_PARAM_CONVERGENT: &str = "convergent";
pub const ATTR_PARAM_COROONLYDESTROYWHENCOMPLETE: &str = "coroonlydestroywhencomplete";
pub const ATTR_PARAM_COROELIDESAFE: &str = "coroelidesafe";
pub const ATTR_PARAM_DEADONRETURN: &str = "dead_on_return";
pub const ATTR_PARAM_DEADONUNWIND: &str = "dead_on_unwind";
pub const ATTR_PARAM_DISABLESANITIZERINSTRUMENTATION: &str = "disablesanitizerinstrumentation";
pub const ATTR_PARAM_FNRETTHUNKEXTERN: &str = "fnretthunkextern";
pub const ATTR_PARAM_HOT: &str = "hot";
pub const ATTR_PARAM_HYBRIDPATCHABLE: &str = "hybridpatchable";
pub const ATTR_PARAM_IMMARG: &str = "immarg";
pub const ATTR_PARAM_INREG: &str = "inreg";
pub const ATTR_PARAM_INLINEHINT: &str = "inlinehint";
pub const ATTR_PARAM_JUMPTABLE: &str = "jumptable";
pub const ATTR_PARAM_MINSIZE: &str = "minsize";
pub const ATTR_PARAM_MUSTPROGRESS: &str = "mustprogress";
pub const ATTR_PARAM_NAKED: &str = "naked";
pub const ATTR_PARAM_NEST: &str = "nest";
pub const ATTR_PARAM_NOALIAS: &str = "noalias";
pub const ATTR_PARAM_NOBUILTIN: &str = "nobuiltin";
pub const ATTR_PARAM_NOCALLBACK: &str = "nocallback";
pub const ATTR_PARAM_NOCFCHECK: &str = "nocfcheck";
pub const ATTR_PARAM_NOCREATEUNDEFORPOISON: &str = "nocreateundeforpoison";
pub const ATTR_PARAM_NODIVERGENCESOURCE: &str = "nodivergencesource";
pub const ATTR_PARAM_NODUPLICATE: &str = "noduplicate";
pub const ATTR_PARAM_NOEXT: &str = "noext";
pub const ATTR_PARAM_NOFREE: &str = "nofree";
pub const ATTR_PARAM_NOIMPLICITFLOAT: &str = "noimplicitfloat";
pub const ATTR_PARAM_NOINLINE: &str = "noinline";
pub const ATTR_PARAM_NOMERGE: &str = "nomerge";
pub const ATTR_PARAM_NOPROFILE: &str = "noprofile";
pub const ATTR_PARAM_NORECURSE: &str = "norecurse";
pub const ATTR_PARAM_NOREDZONE: &str = "noredzone";
pub const ATTR_PARAM_NORETURN: &str = "noreturn";
pub const ATTR_PARAM_NOSANITIZEBOUNDS: &str = "nosanitizebounds";
pub const ATTR_PARAM_NOSANITIZECOVERAGE: &str = "nosanitizecoverage";
pub const ATTR_PARAM_NOSYNC: &str = "nosync";
pub const ATTR_PARAM_NOUNDEF: &str = "noundef";
pub const ATTR_PARAM_NOUNWIND: &str = "nounwind";
pub const ATTR_PARAM_NONLAZYBIND: &str = "nonlazybind";
pub const ATTR_PARAM_NONNULL: &str = "nonnull";
pub const ATTR_PARAM_NULLPOINTERISVALID: &str = "nullpointerisvalid";
pub const ATTR_PARAM_OPTFORFUZZING: &str = "optforfuzzing";
pub const ATTR_PARAM_OPTIMIZEFORDEBUGGING: &str = "optimizefordebugging";
pub const ATTR_PARAM_OPTIMIZEFORSIZE: &str = "optimizeforsize";
pub const ATTR_PARAM_OPTIMIZENONE: &str = "optimizenone";
pub const ATTR_PARAM_PRESPLITCOROUTINE: &str = "presplitcoroutine";
pub const ATTR_PARAM_READNONE: &str = "readnone";
pub const ATTR_PARAM_READONLY: &str = "readonly";
pub const ATTR_PARAM_RETURNED: &str = "returned";
pub const ATTR_PARAM_RETURNSTWICE: &str = "returnstwice";
pub const ATTR_PARAM_SEXT: &str = "sext";
pub const ATTR_PARAM_SAFESTACK: &str = "safestack";
pub const ATTR_PARAM_SANITIZEADDRESS: &str = "sanitizeaddress";
pub const ATTR_PARAM_SANITIZEALLOCTOKEN: &str = "sanitizealloctoken";
pub const ATTR_PARAM_SANITIZEHWADDRESS: &str = "sanitizehwaddress";
pub const ATTR_PARAM_SANITIZEMEMTAG: &str = "sanitizememtag";
pub const ATTR_PARAM_SANITIZEMEMORY: &str = "sanitizememory";
pub const ATTR_PARAM_SANITIZENUMERICALSTABILITY: &str = "sanitizenumericalstability";
pub const ATTR_PARAM_SANITIZEREALTIME: &str = "sanitizerealtime";
pub const ATTR_PARAM_SANITIZEREALTIMEBLOCKING: &str = "sanitizerealtimeblocking";
pub const ATTR_PARAM_SANITIZETHREAD: &str = "sanitizethread";
pub const ATTR_PARAM_SANITIZETYPE: &str = "sanitizetype";
pub const ATTR_PARAM_SHADOWCALLSTACK: &str = "shadowcallstack";
pub const ATTR_PARAM_SKIPPROFILE: &str = "skipprofile";
pub const ATTR_PARAM_SPECULATABLE: &str = "speculatable";
pub const ATTR_PARAM_SPECULATIVELOADHARDENING: &str = "speculativeloadhardening";
pub const ATTR_PARAM_STACKPROTECT: &str = "stackprotect";
pub const ATTR_PARAM_STACKPROTECTREQ: &str = "stackprotectreq";
pub const ATTR_PARAM_STACKPROTECTSTRONG: &str = "stackprotectstrong";
pub const ATTR_PARAM_STRICTFP: &str = "strictfp";
pub const ATTR_PARAM_SWIFTASYNC: &str = "swiftasync";
pub const ATTR_PARAM_SWIFTERROR: &str = "swifterror";
pub const ATTR_PARAM_SWIFTSELF: &str = "swiftself";
pub const ATTR_PARAM_WILLRETURN: &str = "willreturn";
pub const ATTR_PARAM_WRITABLE: &str = "writable";
pub const ATTR_PARAM_WRITEONLY: &str = "writeonly";
pub const ATTR_PARAM_ZEXT: &str = "zeroext";
