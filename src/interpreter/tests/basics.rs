///! Basic tests for instructions/constructions, missing in wabt tests

use std::sync::Arc;
use builder::module;
use elements::{ExportEntry, Internal, ImportEntry, External, GlobalEntry, GlobalType,
	InitExpr, ValueType, Opcodes, Opcode};
use interpreter::Error;
use interpreter::env_native::{env_native_module, UserFunction, UserFunctions, UserFunctionExecutor};
use interpreter::memory::MemoryInstance;
use interpreter::module::{ModuleInstanceInterface, CallerContext, ItemIndex, ExecutionParams};
use interpreter::program::ProgramInstance;
use interpreter::value::RuntimeValue;

#[test]
fn import_function() {
	let module1 = module()
		.with_export(ExportEntry::new("external_func".into(), Internal::Function(0)))
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(3),
				Opcode::End,
			])).build()
			.build()
		.build();

	let module2 = module()
		.with_import(ImportEntry::new("external_module".into(), "external_func".into(), External::Function(0)))
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::Call(0),
				Opcode::I32Const(7),
				Opcode::I32Add,
				Opcode::End,
			])).build()
			.build()
		.build();

	let program = ProgramInstance::new().unwrap();
	let external_module = program.add_module("external_module", module1).unwrap();
	let main_module = program.add_module("main", module2).unwrap();

	assert_eq!(external_module.execute_index(0, vec![].into()).unwrap().unwrap(), RuntimeValue::I32(3));
	assert_eq!(main_module.execute_index(1, vec![].into()).unwrap().unwrap(), RuntimeValue::I32(10));
}

#[test]
fn wrong_import() {
	let side_module = module()
		.with_export(ExportEntry::new("cool_func".into(), Internal::Function(0)))
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I32Const(3),
				Opcode::End,
			])).build()
			.build()
		.build();

	let module = module()
		.with_import(ImportEntry::new("side_module".into(), "not_cool_func".into(), External::Function(0)))
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::Call(0),
				Opcode::I32Const(7),
				Opcode::I32Add,
				Opcode::End,
			])).build()
			.build()
		.build();

	let program = ProgramInstance::new().unwrap();
	let _side_module_instance = program.add_module("side_module", side_module).unwrap();
	let module_instance = program.add_module("main", module).unwrap();

	assert!(module_instance.execute_index(1, vec![].into()).is_err());	
}

#[test]
fn global_get_set() {
	let module = module()
		.with_global(GlobalEntry::new(GlobalType::new(ValueType::I32, true), InitExpr::new(vec![Opcode::I32Const(42)])))
		.with_global(GlobalEntry::new(GlobalType::new(ValueType::I32, false), InitExpr::new(vec![Opcode::I32Const(777)])))
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::GetGlobal(0),
				Opcode::I32Const(8),
				Opcode::I32Add,
				Opcode::SetGlobal(0),
				Opcode::GetGlobal(0),
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::GetGlobal(1),
				Opcode::I32Const(8),
				Opcode::I32Add,
				Opcode::SetGlobal(1),
				Opcode::GetGlobal(1),
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::I64Const(8),
				Opcode::SetGlobal(0),
				Opcode::GetGlobal(0),
				Opcode::End,
			])).build()
			.build()
		.build();

	let program = ProgramInstance::new().unwrap();
	let module = program.add_module("main", module).unwrap();
	assert_eq!(module.execute_index(0, vec![].into()).unwrap().unwrap(), RuntimeValue::I32(50));
	assert_eq!(module.execute_index(1, vec![].into()).unwrap_err(), Error::Variable("trying to update immutable variable".into()));
	assert_eq!(module.execute_index(2, vec![].into()).unwrap_err(), Error::Variable("trying to update variable of type I32 with value of type Some(I64)".into()));
}

#[test]
fn single_program_different_modules() {
	// user function executor
	struct FunctionExecutor {
		pub memory: Arc<MemoryInstance>,
		pub values: Vec<i32>,
	}

	impl UserFunctionExecutor for FunctionExecutor {
		fn execute(&mut self, name: &str, context: CallerContext) -> Result<Option<RuntimeValue>, Error> {
			match name {
				"add" => {
					let memory_value = self.memory.get(0, 1).unwrap()[0];
					let fn_argument = context.value_stack.pop_as::<u32>().unwrap() as u8;
					let sum = memory_value + fn_argument;
					self.memory.set(0, &vec![sum]).unwrap();
					self.values.push(sum as i32);
					Ok(Some(RuntimeValue::I32(sum as i32)))
				},
				"sub" => {
					let memory_value = self.memory.get(0, 1).unwrap()[0];
					let fn_argument = context.value_stack.pop_as::<u32>().unwrap() as u8;
					let diff = memory_value - fn_argument;
					self.memory.set(0, &vec![diff]).unwrap();
					self.values.push(diff as i32);
					Ok(Some(RuntimeValue::I32(diff as i32)))
				},
				_ => Err(Error::Trap("not implemented".into())),
			}
		}
	}

	// create new program
	let program = ProgramInstance::new().unwrap();
	// => env module is created
	let env_instance = program.module("env").unwrap();
	// => linear memory is created
	let env_memory = env_instance.memory(ItemIndex::Internal(0)).unwrap();

	let module = module()
		.with_import(ImportEntry::new("env".into(), "add".into(), External::Function(0)))
		.with_import(ImportEntry::new("env".into(), "sub".into(), External::Function(0)))
		.function()
			.signature().param().i32().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::GetLocal(0),
				Opcode::Call(0),
				Opcode::End,
			])).build()
			.build()
		.function()
			.signature().param().i32().return_type().i32().build()
			.body().with_opcodes(Opcodes::new(vec![
				Opcode::GetLocal(0),
				Opcode::Call(1),
				Opcode::End,
			])).build()
			.build()
		.build();

	// load module
	let module_instance = program.add_module("main", module).unwrap();

	let mut executor = FunctionExecutor {
		memory: env_memory.clone(),
		values: Vec::new(),
	};
	{
		// create native env module with native add && sub implementations
		let functions: UserFunctions = UserFunctions {
			executor: &mut executor,
			functions: vec![UserFunction {
				name: "add".into(),
				params: vec![ValueType::I32],
				result: Some(ValueType::I32),
			}, UserFunction {
				name: "sub".into(),
				params: vec![ValueType::I32],
				result: Some(ValueType::I32),
			}],
		};
		let native_env_instance = Arc::new(env_native_module(env_instance, functions).unwrap());

		// execute functions
		let params = ExecutionParams::with_external("env".into(), native_env_instance);

		assert_eq!(module_instance.execute_index(2, params.clone().add_argument(RuntimeValue::I32(7))).unwrap().unwrap(), RuntimeValue::I32(7));
		assert_eq!(module_instance.execute_index(2, params.clone().add_argument(RuntimeValue::I32(50))).unwrap().unwrap(), RuntimeValue::I32(57));
		assert_eq!(module_instance.execute_index(3, params.clone().add_argument(RuntimeValue::I32(15))).unwrap().unwrap(), RuntimeValue::I32(42));
	}

	assert_eq!(executor.memory.get(0, 1).unwrap()[0], 42);
	assert_eq!(executor.values, vec![7, 57, 42]);
}
