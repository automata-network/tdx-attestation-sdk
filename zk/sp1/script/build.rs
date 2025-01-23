use sp1_build::{build_program_with_args, BuildArgs};
use sp1_sdk::SP1_CIRCUIT_VERSION;

fn main() {
    build_program_with_args(
        "../program",
        BuildArgs {
            output_directory: Some("../elf".to_string()),
            elf_name: Some("dcap-sp1-guest-program-elf".to_string()),
            docker: true,
            tag: SP1_CIRCUIT_VERSION.to_string(),
            ..Default::default()
        },
    )
}