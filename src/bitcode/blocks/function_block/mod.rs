//! Function Block

pub(in crate::bitcode) mod inst_alloca;
pub(in crate::bitcode) mod inst_atomicrmw;
pub(in crate::bitcode) mod inst_binop;
pub(in crate::bitcode) mod inst_br;
pub(in crate::bitcode) mod inst_call;
pub(in crate::bitcode) mod inst_cast;
pub(in crate::bitcode) mod inst_cmp;
pub(in crate::bitcode) mod inst_cmpxchg;
pub(in crate::bitcode) mod inst_extractelt;
pub(in crate::bitcode) mod inst_extractval;
pub(in crate::bitcode) mod inst_fence;
pub(in crate::bitcode) mod inst_freeze;
pub(in crate::bitcode) mod inst_gep;
pub(in crate::bitcode) mod inst_insertelt;
pub(in crate::bitcode) mod inst_insertval;
pub(in crate::bitcode) mod inst_invoke;
pub(in crate::bitcode) mod inst_landingpad;
pub(in crate::bitcode) mod inst_load;
pub(in crate::bitcode) mod inst_loadatomic;
pub(in crate::bitcode) mod inst_phi;
pub(in crate::bitcode) mod inst_resume;
pub(in crate::bitcode) mod inst_ret;
pub(in crate::bitcode) mod inst_shufflevec;
pub(in crate::bitcode) mod inst_store;
pub(in crate::bitcode) mod inst_storeatomic;
pub(in crate::bitcode) mod inst_switch;
pub(in crate::bitcode) mod inst_unop;
pub(in crate::bitcode) mod inst_unreachble;
pub(in crate::bitcode) mod inst_vselect;
pub(in crate::bitcode) mod utils;

// INST_GEP_OLD = 4, // [n x operands]
// INST_SELECT = 5, // [ty, opval, opval, opval]
// INST_CMP = 9, // [opty, opval, opval, pred]
// // 14 is unused.
// // 17 is unused.
// // 18 is unused.
// // 21 is unused.
// // 22 is unused.
// INST_VAARG = 23, // [valistty, valist, instty]
// // This store code encodes the pointer type, rather than the value type
// // this is so information only available in the pointer type (e.g. address
// // spaces) is retained.
// INST_STORE_OLD = 24, // [ptrty,ptr,val, align, vol]
// // 25 is unused.
// INST_INBOUNDS_GEP_OLD = 30, // [n x operands]
// INST_INDIRECTBR = 31, // [opty, op0, op1, ...]
// // 32 is unused.
// DEBUG_LOC_AGAIN = 33,
// DEBUG_LOC = 35,  // [Line, Col, ScopeVal, IAVal]
// INST_CMPXCHG_OLD = 37, // [ptrty, ptr, cmp, val, vol, ordering, syncscope, failure_ordering?, weak?]
// INST_ATOMICRMW_OLD = 38, // [ptrty,ptr,val, operation, align, vol, ordering, syncscope]
// INST_LANDINGPAD_OLD = 40, // [ty, val, val, num, id0, val0...]
// INST_CLEANUPRET = 48, // [val] or [val,bb#]
// INST_CATCHRET = 49, // [val, bb#]
// INST_CATCHPAD = 50, // [bb#,bb#,num,args...]
// INST_CLEANUPPAD = 51, // [num, args...]
// INST_CATCHSWITCH = 52, // [num,args...] or [num,args...,bb]
// // 53 is unused.
// // 54 is unused.
// OPERAND_BUNDLE = 55, // [tag#, value...]
// INST_CALLBR = 57, // [attr, cc, norm, transfs, fnty, fnid, args...]
// BLOCKADDR_USERS = 60, // [value...]
// DEBUG_RECORD_VALUE = 61, // [DILocation, DILocalVariable, DIExpression, ValueAsMetadata]
// DEBUG_RECORD_DECLARE = 62, // [DILocation, DILocalVariable, DIExpression, ValueAsMetadata]
// DEBUG_RECORD_ASSIGN = 63, // [DILocation, DILocalVariable, DIExpression, ValueAsMetadata, DIAssignID, DIExpression (addr), ValueAsMetadata (addr)]
// DEBUG_RECORD_VALUE_SIMPLE = 64, // [DILocation, DILocalVariable, DIExpression, Value]
// DEBUG_RECORD_LABEL = 65, // [DILocation, DILabel]
// DEBUG_RECORD_DECLARE_VALUE = 66, // [DILocation, DILocalVariable, DIExpression, ValueAsMetadata]
