use clap::Parser;

// The ELF64 sizes are the same for Half and Word
const ELF32_HALF: usize = 2;
const ELF_LITTLE_ENDIAN: u8 = 0x01;

#[derive(Parser)]
#[command(name = "elf_machine_updater")]
#[command(bin_name = "elf_machine_updater")]
enum Cli {
    ReadMachine(ReadMachineArgs),
    UpdateMachine(UpdateMachineArgs),
}

#[derive(clap::Args)]
#[command(version, about, long_about = None)]
struct ReadMachineArgs {
    #[arg(long)]
    elf: std::path::PathBuf,
}

#[derive(clap::Args)]
#[command(version, about, long_about = None)]
struct UpdateMachineArgs {
    #[arg(long)]
    elf: std::path::PathBuf,
    #[arg(long)]
    value: u16,
    #[arg(long)]
    dry_run: bool, // default is false
}

fn main() {
    match Cli::parse() {
        Cli::ReadMachine(args) => {
            let payload = read_file(&args.elf);
            let machine = get_machine(&payload);
            println!("Machine is '{}'", machine);
        }
        Cli::UpdateMachine(args) => {
            let payload = read_file(&args.elf);
            let old_machine = get_machine(&payload);
            println!("Old machine is '{}'", old_machine);

            // Do the magic
            let updated_payload = update_machine(payload, args.value);

            if !args.dry_run {
                use std::io::prelude::*;

                println!("Updating file...");
                let mut file = std::fs::OpenOptions::new()
                    .write(true)
                    .create(false)
                    .truncate(true)
                    .open(args.elf)
                    .expect("Cannot open file");
                let _ = file
                    .write_all(&updated_payload)
                    .expect("Updating payload has failed");
            } else {
                println!("Not updating file because dry run was activated");
            }

            // Check
            let new_machine = get_machine(&updated_payload);
            println!("New machine is '{}'", new_machine);
        }
    }
}

fn read_file(path: &std::path::PathBuf) -> Vec<u8> {
    std::fs::read(path).expect("Could not read file.")
}

fn get_machine(payload: &[u8]) -> u16 {
    // Check the magic numbers of payload
    assert_eq!(payload[0], 0x7F);
    assert_eq!(payload[1], 0x45);
    assert_eq!(payload[2], 0x4C);

    // We know that.
    let endian_offset = 0x05;
    let machine_offset = 0x12;

    // Get the slice of the file but only for the machine.
    let machine_slice: &[u8; ELF32_HALF] = &payload[machine_offset..(machine_offset + ELF32_HALF)]
        .try_into()
        .unwrap();

    let is_little_endian = payload[endian_offset] == ELF_LITTLE_ENDIAN;

    // The endian is specified in the ELF file.
    let machine_num = if is_little_endian {
        u16::from_le_bytes(machine_slice.clone())
    } else {
        u16::from_be_bytes(machine_slice.clone())
    };

    machine_num
}

/// Update the machine value in the payload with the given value and return the updated payload.
fn update_machine(mut payload: Vec<u8>, value: u16) -> Vec<u8> {
    // Check the magic numbers of payload
    assert_eq!(payload[0], 0x7F);
    assert_eq!(payload[1], 0x45);
    assert_eq!(payload[2], 0x4C);

    // We know that.
    let endian_offset = 0x05;
    let machine_offset = 0x12;

    // Get the slice of the file but only for the machine.
    //let machine_slice : &[u8; ELF32_Half] = &payload[machine_offset..(machine_offset + ELF32_Half)].try_into().unwrap();

    let is_little_endian = payload[endian_offset] == ELF_LITTLE_ENDIAN;

    // The endian is specified in the ELF file.
    let machine_num = if is_little_endian {
        u16::to_le_bytes(value)
    } else {
        u16::to_be_bytes(value)
    };

    for (index, offset) in (machine_offset..(machine_offset + 2)).enumerate() {
        payload[offset] = machine_num[index];
    }

    payload
}
