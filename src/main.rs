use std::env::args;
use std::fs;
use std::process::ExitCode;
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

	let shader = YuriShader::new(&input)
		.unwrap();

	ExitCode::SUCCESS
}
