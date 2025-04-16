extern crate core;

use std::env::args;
use std::fs;
use std::process::ExitCode;
use yuri::lex::YuriTokenType;
use yuri::YuriShader;

fn main() -> ExitCode {
	let args = args()
		.collect::<Vec<String>>();

	let input_path = if let Some(arg) = args.get(1) {
		arg
	} else {
		eprintln!("Must provide a file as an argument");
		return ExitCode::FAILURE;
	};

    // TODO: frontend; just read the arguments and spit out some SPIR-V somewhere.
	let input = fs::read_to_string(input_path)
		.expect("failed to read file");

	let ast = YuriShader::lex(&input).unwrap();
	let ast_string_errors: String = ast.iter()
		.filter(|tok| match tok.token_type { YuriTokenType::Unknown(_) => true, _ => false })
		.map(|tok| format!("{tok:?}"))
		.collect::<Vec<String>>()
		.join("\n");
	let ast_string: String = ast.iter()
		.filter(|tok| match tok.token_type { YuriTokenType::Unknown(_) => false, _ => true })
		.map(|tok| format!("{tok:?}"))
		.collect::<Vec<String>>()
		.join("\n");
	println!("AST:\n{ast_string}");
	println!("errors:\n{ast_string_errors}");

	let shader = YuriShader::parse(&ast)
		.unwrap();


	ExitCode::SUCCESS
}
