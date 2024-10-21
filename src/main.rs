mod types;

use std::fs;

use crate::types::{
    from_compiled_module_def, from_compiled_script_def, to_compiled_module_def,
    to_compiled_script_def, CompiledModuleDef, CompiledScriptDef,
};
use clap::{Args, Parser, Subcommand};
use move_binary_format::file_format::{CompiledModule, CompiledScript};
use move_bytecode_verifier::{
    verify_module_with_config, verify_script_with_config, VerifierConfig,
};
use serde::{Deserialize, Serialize};
use serde_json;


#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Disassemble script bytecode
    DisScript(DisassembleCommandArgs),
    /// Disassemble module bytecode
    DisModule(DisassembleCommandArgs),
    /// Assemble script bytecode
    AsmScript(AssembleCommandArgs),
    /// Assemble module bytecode
    AsmModule(AssembleCommandArgs),
    /// Verify script bytecode
    VerifyScript(VerifyCommandArgs),
    /// Verify module bytecode
    VerifyModule(VerifyCommandArgs),
}

#[derive(Args)]
struct DisassembleCommandArgs {
    /// The path to the bytecode file to disassemble
    #[clap(required = true)]
    pub bytecode_file_path: String,
    /// The path to the disassembled file to output
    #[clap(short = 'o', long = "output-file-path")]
    pub output_file_path: Option<String>,
}

#[derive(Args)]
struct AssembleCommandArgs {
    /// The path to the disassembled file to assemble
    #[clap(required = true)]
    pub disassembled_file_path: String,
    /// The path to the assembled file to output
    #[clap(required = true, short = 'o', long = "output-file-path")]
    pub output_file_path: String,
}

#[derive(Args)]
struct VerifyCommandArgs {
    /// The path to the bytecode file to disassemble
    #[clap(required = true)]
    pub bytecode_file_path: String,
}


fn disassemble_bytecode_file_to_output(
    bytecode_file_path: &String,
    output_file_path: &Option<String>,
    is_script: bool,
) {
    let disassembled: String;
    let binary = fs::read(&bytecode_file_path).expect("Unable to read bytecode file");

    if is_script {
        let script = CompiledScript::deserialize(&binary)
            .expect("Unable to deserialize script bytecode file");
        disassembled = serde_json::to_string_pretty(&to_compiled_script_def(&script))
            .expect("Fail to serde script to json");
    } else {
        let module = CompiledModule::deserialize(&binary)
            .expect("Unable to deserialize module bytecode file");
        disassembled = serde_json::to_string_pretty(&to_compiled_module_def(&module))
            .expect("Fail to serde module to json");
    }

    if let Some(output_file_path) = &output_file_path {
        fs::write(output_file_path, disassembled).expect("Fail to write to file");
    } else {
        println!("{}", disassembled);
    }
}

fn assemble_bytecode_file_to_output(
    disassembled_file_path: &String,
    output_file_path: &String,
    is_script: bool,
) {
    let mut binary: Vec<u8> = vec![];
    let disassembled =
        fs::read_to_string(disassembled_file_path).expect("Unable to read bytecode file");

    if is_script {
        let script_def: CompiledScriptDef = serde_json::from_str(disassembled.as_str())
            .expect("Fail to deserialize the disassembled data");
        let script = from_compiled_script_def(&script_def);
        script
            .serialize(&mut binary)
            .expect("Fail to serialize CompiledScript to binary");
    } else {
        let module_def: CompiledModuleDef = serde_json::from_str(disassembled.as_str())
            .expect("Fail to deserialize the disassembled data");
        let module = from_compiled_module_def(&module_def);
        module
            .serialize(&mut binary)
            .expect("Fail to serialize CompiledModule to binary");
    }

    fs::write(output_file_path, binary).expect("Fail to write binary to output file");
}

fn verify_bytecode_file(bytecode_file_path: &String, is_script: bool) {
    let binary = fs::read(bytecode_file_path).expect("Unable to read bytecode file");

    if is_script {
        let script = CompiledScript::deserialize(&binary)
            .expect("Unable to deserialize script bytecode file");
        let res = verify_script_with_config(&VerifierConfig::default(), &script);
        println!("{:?}", res);
    } else {
        let module = CompiledModule::deserialize(&binary)
            .expect("Unable to deserialize script bytecode file");
        let res = verify_module_with_config(&VerifierConfig::default(), &module);
        println!("{:?}", res);
    }

}


fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::DisScript(args) => {
            disassemble_bytecode_file_to_output(
                &args.bytecode_file_path,
                &args.output_file_path,
                true,
            );
        }
        Commands::DisModule(args) => {
            disassemble_bytecode_file_to_output(
                &args.bytecode_file_path,
                &args.output_file_path,
                false,
            );
        }
        Commands::AsmScript(args) => {
            assemble_bytecode_file_to_output(
                &args.disassembled_file_path,
                &args.output_file_path,
                true,
            )
        }
        Commands::AsmModule(args) => {
            assemble_bytecode_file_to_output(
                &args.disassembled_file_path,
                &args.output_file_path,
                false,
            )
        }
        Commands::VerifyScript(args) => {
            verify_bytecode_file(&args.bytecode_file_path, true);
        }
        Commands::VerifyModule(args) => {
            verify_bytecode_file(&args.bytecode_file_path, false);
        }
    };
}
