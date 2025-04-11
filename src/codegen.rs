use std::fs;
use dr::Operand;
use rspirv::binary::Assemble;
use rspirv::binary::Disassemble;
use rspirv::spirv::ExecutionMode;
use rspirv::{dr, spirv};
use std::mem::transmute;

fn codegen() {
	// Building
	let mut b = dr::Builder::new();
	b.set_version(1, 0);
	b.capability(spirv::Capability::Shader);
	b.memory_model(spirv::AddressingModel::Logical, spirv::MemoryModel::GLSL450);
	// typedefs
	let unit = b.type_void();
	let f = b.type_float(32);
	let f4 = b.type_vector(f, 4);
	let f2 = b.type_vector(f, 2);
	let void_func = b.type_function(unit, vec![]);
	let out_f4 = b.type_pointer(None, spirv::StorageClass::Output, f4);
	let in_f2 = b.type_pointer(None, spirv::StorageClass::Input, f2);

	let position = b.variable(in_f2, None, spirv::StorageClass::Input, None);
	b.decorate(
		position,
		spirv::Decoration::Location,
		[Operand::LiteralBit32(0)],
	);
	b.name(position, "position");

	let tex_coord = b.variable(in_f2, None, spirv::StorageClass::Input, None);
	b.decorate(
		tex_coord,
		spirv::Decoration::Location,
		[Operand::LiteralBit32(1)],
	);
	b.name(tex_coord, "tex_coord");

	let frag_color = b.variable(out_f4, None, spirv::StorageClass::Output, None);
	b.decorate(
		frag_color,
		spirv::Decoration::Location,
		[Operand::LiteralBit32(0)],
	);
	b.name(frag_color, "frag_color");

	let f_0 = b.constant_bit32(f, 0);
	let f_1 = b.constant_bit32(f, unsafe { transmute(1.0f32) });

	let f2_lit_0_1 = b.constant_composite(f2, [f_0, f_1]);

	// entry point function
	let entry = {
		let entry = b
			.begin_function(
				unit,
				None,
				spirv::FunctionControl::NONE,
				// spirv::FunctionControl::DONT_INLINE | spirv::FunctionControl::CONST,
				void_func,
			)
			.unwrap();
		b.begin_block(None).unwrap();

		// %14 = OpLoad %v2float %texCoord
		let tex_coord_local = b.load(f2, None, tex_coord, None, None).unwrap();

		// %17 = OpCompositeExtract %float %14 0
		// %18 = OpCompositeExtract %float %14 1
		// %19 = OpCompositeConstruct %v4float %17 %18 %float_0 %float_1
		let out = b
			.composite_construct(f4, None, [tex_coord_local, f2_lit_0_1])
			.unwrap();

		// OpStore %fragColor %19
		b.store(frag_color, out, None, None).unwrap();

		b.ret().unwrap();
		b.end_function().unwrap();
		entry
	};
	b.entry_point(
		spirv::ExecutionModel::Fragment,
		entry,
		"main",
		[
			// outputs
			frag_color, // inputs
			tex_coord,
		],
	);
	// I get that an upper-left coordinate system is common,
	// but we're not using it.
	b.execution_mode(entry, ExecutionMode::OriginLowerLeft, []);
	let module = b.module();

	// Assembling
	let code = module.assemble();
	assert!(code.len() > 20); // Module header contains 5 words
	assert_eq!(spirv::MAGIC_NUMBER, code[0]);

	// Parsing
	let mut loader = dr::Loader::new();
	rspirv::binary::parse_words(&code, &mut loader).unwrap();
	let module = loader.module();

	let assembled = module.assemble();
	let disassembled = module.disassemble();
	println!("~~~~~ disassembly ~~~~~");
	println!("{disassembled}");

	fs::write(
		"yuri.frag.spvasm",
		disassembled
	).unwrap();


	let disassembled_bytes: Vec<u8> = assembled.iter().flat_map(|w| {
		w.to_le_bytes()
	}).collect();

	fs::write(
		"yuri.frag.spv",
		disassembled_bytes
	).unwrap();
}