#![cfg_attr(not(test), no_std, no_main)]
use core::panic::PanicInfo;
use wa::{Arg, Command, Subcommand};

fn cli() -> Command {
    Command::new("we", "A modern vcs").subcommand(
        Subcommand::new("commit", "Commit changes").arg(
            Arg::new("message", "The commit message")
                .short("m")
                .long("message")
                .required(true),
        ),
    )
}
#[unsafe(no_mangle)]
pub extern "C" fn _start(argc: isize, argv: *const *const u8) -> ! {
    let mut args = [""; 8];
    for i in 0..argc {
        let arg =
            unsafe { core::ffi::CStr::from_ptr(argv.wrapping_add(i as usize).read() as *const i8) };
        let arg_str = arg.to_str().unwrap();
        args[i as usize] = arg_str;
    }
    let cmd = cli().get_matches(&args);

    match cmd.subcommand_name {
        Some("commit") => {
            let message = cmd.values[0].unwrap();
            panic!("Commit message: {} {}", message.0, message.1);
        }
        _ => {
            // Gérer les autres sous-commandes ou afficher un message d'erreur
        }
    }
    loop {}
}
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
