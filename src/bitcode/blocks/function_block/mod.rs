//! Function Block

pub(in crate::bitcode) mod inst_alloca;
pub(in crate::bitcode) mod inst_binop;
pub(in crate::bitcode) mod inst_br;
pub(in crate::bitcode) mod inst_call;
pub(in crate::bitcode) mod inst_cast;
pub(in crate::bitcode) mod inst_cmp;
pub(in crate::bitcode) mod inst_extractelt;
pub(in crate::bitcode) mod inst_gep;
pub(in crate::bitcode) mod inst_insertelt;
pub(in crate::bitcode) mod inst_load;
pub(in crate::bitcode) mod inst_phi;
pub(in crate::bitcode) mod inst_ret;
pub(in crate::bitcode) mod inst_shufflevec;
pub(in crate::bitcode) mod inst_store;
pub(in crate::bitcode) mod inst_vselect;
pub(in crate::bitcode) mod utils;

// INST_GEP_OLD = 4, // [n x operands]
// INST_SELECT = 5, // [ty, opval, opval, opval]
// INST_CMP = 9, // [opty, opval, opval, pred]
// INST_SWITCH = 12, // [opty, op0, op1, ...]
// INST_INVOKE = 13, // [attr, fnty, op0,op1, ...]
// // 14 is unused.
// INST_UNREACHABLE = 15,
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
// INST_EXTRACTVAL = 26, // [n x operands]
// INST_INSERTVAL = 27, // [n x operands]
// INST_INBOUNDS_GEP_OLD = 30, // [n x operands]
// INST_INDIRECTBR = 31, // [opty, op0, op1, ...]
// // 32 is unused.
// DEBUG_LOC_AGAIN = 33,
// DEBUG_LOC = 35,  // [Line, Col, ScopeVal, IAVal]
// INST_FENCE = 36, // [ordering, syncscope]
// INST_CMPXCHG_OLD = 37, // [ptrty, ptr, cmp, val, vol, ordering, syncscope, failure_ordering?, weak?]
// INST_ATOMICRMW_OLD = 38, // [ptrty,ptr,val, operation, align, vol, ordering, syncscope]
// INST_RESUME = 39, // RESUME: [opval]
// INST_LANDINGPAD_OLD = 40, // [ty, val, val, num, id0, val0...]
// INST_LOADATOMIC = 41, // [opty, op, align, vol, ordering, syncscope]
// INST_STOREATOMIC_OLD = 42, // [ptrty, ptr,val, align, vol, ordering, syncscope]
// INST_STOREATOMIC = 45, // [ptrty, ptr, val, align, vol]
// INST_CMPXCHG = 46, // [ptrty, ptr, cmp, val, vol, success_ordering, syncscope, failure_ordering, weak]
// INST_LANDINGPAD = 47, // [ty,val,num,id0,val0...]
// INST_CLEANUPRET = 48, // [val] or [val,bb#]
// INST_CATCHRET = 49, // [val, bb#]
// INST_CATCHPAD = 50, // [bb#,bb#,num,args...]
// INST_CLEANUPPAD = 51, // [num, args...]
// INST_CATCHSWITCH = 52, // [num,args...] or [num,args...,bb]
// // 53 is unused.
// // 54 is unused.
// OPERAND_BUNDLE = 55, // [tag#, value...]
// INST_UNOP = 56, // [opcode, ty, opval]
// INST_CALLBR = 57, // [attr, cc, norm, transfs, fnty, fnid, args...]
// INST_FREEZE = 58, // [opty, opval]
// INST_ATOMICRMW = 59, // [ptrty, ptr, valty, val, operation, align, vol, ordering, syncscope]
// BLOCKADDR_USERS = 60, // [value...]
// DEBUG_RECORD_VALUE = 61, // [DILocation, DILocalVariable, DIExpression, ValueAsMetadata]
// DEBUG_RECORD_DECLARE = 62, // [DILocation, DILocalVariable, DIExpression, ValueAsMetadata]
// DEBUG_RECORD_ASSIGN = 63, // [DILocation, DILocalVariable, DIExpression, ValueAsMetadata, DIAssignID, DIExpression (addr), ValueAsMetadata (addr)]
// DEBUG_RECORD_VALUE_SIMPLE = 64, // [DILocation, DILocalVariable, DIExpression, Value]
// DEBUG_RECORD_LABEL = 65, // [DILocation, DILabel]
// DEBUG_RECORD_DECLARE_VALUE = 66, // [DILocation, DILocalVariable, DIExpression, ValueAsMetadata]
