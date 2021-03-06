use std::sync::{Arc, Weak};

use builder::module;
use elements::{Module, FunctionType, ExportEntry, Internal, GlobalEntry, GlobalType,
	ValueType, InitExpr, Opcode, Opcodes};
use interpreter::Error;
use interpreter::env_native::NATIVE_INDEX_FUNC_MIN;
use interpreter::module::{ModuleInstanceInterface, ModuleInstance, ExecutionParams,
	ItemIndex, CallerContext};
use interpreter::memory::{MemoryInstance, LINEAR_MEMORY_PAGE_SIZE};
use interpreter::table::TableInstance;
use interpreter::value::{RuntimeValue, TransmuteInto};
use interpreter::variable::VariableInstance;

/// Memory address, at which stack begins.
const DEFAULT_STACK_BASE: u32 = 0;
/// Memory, allocated for stack.
const DEFAULT_TOTAL_STACK: u32 = 5 * 1024 * 1024;
/// Total memory, allocated by default.
const DEFAULT_TOTAL_MEMORY: u32 = 16 * 1024 * 1024;
/// Whether memory can be enlarged, or not.
const DEFAULT_ALLOW_MEMORY_GROWTH: bool = false;
/// Default tableBase variable value.
const DEFAULT_TABLE_BASE: u32 = 0;

/// Defaul table size.
const DEFAULT_TABLE_SIZE: u32 = 16;

/// Index of default memory.
const INDEX_MEMORY: u32 = 0;

/// Index of default table.
const INDEX_TABLE: u32 = 0;

/// Index of STACK_BASE global variable.
const INDEX_GLOBAL_STACK_BASE: u32 = 0;
/// Index of STACK_TOP global variable.
const INDEX_GLOBAL_STACK_TOP: u32 = 1;
/// Index of STACK_MAX global variable.
const INDEX_GLOBAL_STACK_MAX: u32 = 2;
/// Index of DYNAMIC_BASE global variable.
const INDEX_GLOBAL_DYNAMIC_BASE: u32 = 3;
/// Index of DYNAMICTOP_PTR global variable.
const INDEX_GLOBAL_DYNAMICTOP_PTR: u32 = 4;
/// Index of TOTAL_MEMORY global variable.
const INDEX_GLOBAL_TOTAL_MEMORY: u32 = 5;
/// Index of ABORT global variable.
const INDEX_GLOBAL_ABORT: u32 = 6;
/// Index of EXITSTATUS global variable.
const INDEX_GLOBAL_EXIT_STATUS: u32 = 7;
/// Index of tableBase global variable.
const INDEX_GLOBAL_TABLE_BASE: u32 = 8;

/// Index of abort function.
const INDEX_FUNC_ABORT: u32 = 0;
/// Index of assert function.
const INDEX_FUNC_ASSERT: u32 = 1;
/// Index of enlargeMemory function.
const INDEX_FUNC_ENLARGE_MEMORY: u32 = 2;
/// Index of getTotalMemory function.
const INDEX_FUNC_GET_TOTAL_MEMORY: u32 = 3;
/// Min index of reserver function.
const INDEX_FUNC_MIN_NONUSED: u32 = 4;
/// Max index of reserved function.
const INDEX_FUNC_MAX: u32 = NATIVE_INDEX_FUNC_MIN - 1;

/// Environment parameters.
pub struct EnvParams {
	/// Stack size in bytes.
	pub total_stack: u32,
	/// Total memory size in bytes.
	pub total_memory: u32,
	/// Allow memory growth.
	pub allow_memory_growth: bool,
}

pub struct EnvModuleInstance {
	_params: EnvParams,
	instance: ModuleInstance,
}

impl EnvModuleInstance {
	pub fn new(params: EnvParams, module: Module) -> Result<Self, Error> {
		let instance = ModuleInstance::new(Weak::default(), module)?;

		Ok(EnvModuleInstance {
			_params: params,
			instance: instance,
		})
	}
}

impl ModuleInstanceInterface for EnvModuleInstance {
	fn execute_main(&self, params: ExecutionParams) -> Result<Option<RuntimeValue>, Error> {
		self.instance.execute_main(params)
	}

	fn execute_index(&self, index: u32, params: ExecutionParams) -> Result<Option<RuntimeValue>, Error> {
		self.instance.execute_index(index, params)
	}

	fn execute_export(&self, name: &str, params: ExecutionParams) -> Result<Option<RuntimeValue>, Error> {
		self.instance.execute_export(name, params)
	}

	fn export_entry(&self, name: &str) -> Result<Internal, Error> {
		self.instance.export_entry(name)
	}

	fn table(&self, index: ItemIndex) -> Result<Arc<TableInstance>, Error> {
		self.instance.table(index)
	}

	fn memory(&self, index: ItemIndex) -> Result<Arc<MemoryInstance>, Error> {
		self.instance.memory(index)
	}

	fn global(&self, index: ItemIndex) -> Result<Arc<VariableInstance>, Error> {
		self.instance.global(index)
	}

	fn call_function(&self, outer: CallerContext, index: ItemIndex) -> Result<Option<RuntimeValue>, Error> {
		self.instance.call_function(outer, index)
	}

	fn call_function_indirect(&self, outer: CallerContext, table_index: ItemIndex, type_index: u32, func_index: u32) -> Result<Option<RuntimeValue>, Error> {
		self.instance.call_function_indirect(outer, table_index, type_index, func_index)
	}

	fn call_internal_function(&self, outer: CallerContext, index: u32, _function_type: Option<&FunctionType>) -> Result<Option<RuntimeValue>, Error> {
		// TODO: check function type
		// to make interpreter independent of *SCRIPTEN runtime, just make abort/assert = interpreter Error
		match index {
			INDEX_FUNC_ABORT => self.global(ItemIndex::IndexSpace(INDEX_GLOBAL_ABORT))
				.and_then(|g| g.set(RuntimeValue::I32(1)))
				.and_then(|_| Err(Error::Trap("abort".into()))),
			INDEX_FUNC_ASSERT => outer.value_stack.pop_as::<i32>()
				.and_then(|condition| if condition == 0 {
					self.global(ItemIndex::IndexSpace(INDEX_GLOBAL_ABORT))
						.and_then(|g| g.set(RuntimeValue::I32(1)))
						.and_then(|_| Err(Error::Trap("assertion failed".into())))
				} else {
					Ok(None)
				}),
			INDEX_FUNC_ENLARGE_MEMORY => Ok(Some(RuntimeValue::I32(0))), // TODO: support memory enlarge
			INDEX_FUNC_GET_TOTAL_MEMORY => self.global(ItemIndex::IndexSpace(INDEX_GLOBAL_TOTAL_MEMORY))
				.map(|g| g.get())
				.map(Some),
			INDEX_FUNC_MIN_NONUSED ... INDEX_FUNC_MAX => Err(Error::Trap("unimplemented".into())),
			_ => Err(Error::Trap(format!("trying to call function with index {} in env module", index))),
		}
	}
}

pub fn env_module(params: EnvParams) -> Result<EnvModuleInstance, Error> {
	debug_assert!(params.total_stack < params.total_memory);
	debug_assert!((params.total_stack % LINEAR_MEMORY_PAGE_SIZE) == 0);
	debug_assert!((params.total_memory % LINEAR_MEMORY_PAGE_SIZE) == 0);
	let builder = module()
		// memory regions
		.memory()
			.with_min(params.total_memory / LINEAR_MEMORY_PAGE_SIZE)
			.with_max(params.max_memory().map(|m| m / LINEAR_MEMORY_PAGE_SIZE))
			.build()
			.with_export(ExportEntry::new("memory".into(), Internal::Memory(INDEX_MEMORY)))
		// tables
		.table()
			.with_min(DEFAULT_TABLE_SIZE)
			.build()
			.with_export(ExportEntry::new("table".into(), Internal::Table(INDEX_TABLE)))
		// globals
		.with_global(GlobalEntry::new(GlobalType::new(ValueType::I32, false), InitExpr::new(vec![Opcode::I32Const(DEFAULT_STACK_BASE.transmute_into())])))
			.with_export(ExportEntry::new("STACK_BASE".into(), Internal::Global(INDEX_GLOBAL_STACK_BASE)))
		.with_global(GlobalEntry::new(GlobalType::new(ValueType::I32, true), InitExpr::new(vec![Opcode::I32Const(DEFAULT_STACK_BASE.transmute_into())])))
			.with_export(ExportEntry::new("STACKTOP".into(), Internal::Global(INDEX_GLOBAL_STACK_TOP)))
		.with_global(GlobalEntry::new(GlobalType::new(ValueType::I32, false), InitExpr::new(vec![Opcode::I32Const((DEFAULT_STACK_BASE + params.total_stack).transmute_into())])))
			.with_export(ExportEntry::new("STACK_MAX".into(), Internal::Global(INDEX_GLOBAL_STACK_MAX)))
		.with_global(GlobalEntry::new(GlobalType::new(ValueType::I32, false), InitExpr::new(vec![Opcode::I32Const((DEFAULT_STACK_BASE + params.total_stack).transmute_into())])))
			.with_export(ExportEntry::new("DYNAMIC_BASE".into(), Internal::Global(INDEX_GLOBAL_DYNAMIC_BASE)))
		.with_global(GlobalEntry::new(GlobalType::new(ValueType::I32, true), InitExpr::new(vec![Opcode::I32Const((DEFAULT_STACK_BASE + params.total_stack).transmute_into())])))
			.with_export(ExportEntry::new("DYNAMICTOP_PTR".into(), Internal::Global(INDEX_GLOBAL_DYNAMICTOP_PTR)))
			.with_global(GlobalEntry::new(GlobalType::new(ValueType::I32, params.allow_memory_growth), InitExpr::new(vec![Opcode::I32Const(params.total_memory.transmute_into())])))
			.with_export(ExportEntry::new("TOTAL_MEMORY".into(), Internal::Global(INDEX_GLOBAL_TOTAL_MEMORY)))
		.with_global(GlobalEntry::new(GlobalType::new(ValueType::I32, true), InitExpr::new(vec![Opcode::I32Const(0)])))
			.with_export(ExportEntry::new("ABORT".into(), Internal::Global(INDEX_GLOBAL_ABORT)))
		.with_global(GlobalEntry::new(GlobalType::new(ValueType::I32, true), InitExpr::new(vec![Opcode::I32Const(0)])))
			.with_export(ExportEntry::new("EXITSTATUS".into(), Internal::Global(INDEX_GLOBAL_EXIT_STATUS)))
		.with_global(GlobalEntry::new(GlobalType::new(ValueType::I32, false), InitExpr::new(vec![Opcode::I32Const(DEFAULT_TABLE_BASE.transmute_into())]))) // TODO: what is this?
			.with_export(ExportEntry::new("tableBase".into(), Internal::Global(INDEX_GLOBAL_TABLE_BASE)))
		// functions
		.function()
			.signature().build()
			.body().with_opcodes(Opcodes::new(vec![Opcode::Unreachable, Opcode::End])).build()
			.build()
			.with_export(ExportEntry::new("abort".into(), Internal::Function(INDEX_FUNC_ABORT)))
		.function()
			.signature().param().i32().build()
			.body().with_opcodes(Opcodes::new(vec![Opcode::Unreachable, Opcode::End])).build()
			.build()
			.with_export(ExportEntry::new("assert".into(), Internal::Function(INDEX_FUNC_ASSERT)))
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![Opcode::Unreachable, Opcode::End])).build()
			.build()
			.with_export(ExportEntry::new("enlargeMemory".into(), Internal::Function(INDEX_FUNC_ENLARGE_MEMORY)))
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![Opcode::Unreachable, Opcode::End])).build()
			.build()
			.with_export(ExportEntry::new("getTotalMemory".into(), Internal::Function(INDEX_FUNC_GET_TOTAL_MEMORY)));

	EnvModuleInstance::new(params, builder.build())
}

impl Default for EnvParams {
	fn default() -> Self {
		EnvParams {
			total_stack: DEFAULT_TOTAL_STACK,
			total_memory: DEFAULT_TOTAL_MEMORY,
			allow_memory_growth: DEFAULT_ALLOW_MEMORY_GROWTH,
		}
	}
}

impl EnvParams {
	fn max_memory(&self) -> Option<u32> {
		if self.allow_memory_growth { None } else { Some(self.total_memory) }
	}
}
