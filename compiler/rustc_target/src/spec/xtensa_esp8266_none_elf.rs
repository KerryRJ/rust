use crate::spec::{LinkerFlavor, PanicStrategy, Target, TargetOptions, RelocModel};
use crate::abi::Endian;

pub fn target() -> Target {
    Target {
        llvm_target: "xtensa".to_string(),
        pointer_width: 32,
        data_layout: "e-m:e-p:32:32-i8:8:32-i16:16:32-i64:64-n32".to_string(),
        arch: "xtensa".to_string(),
        
        options: TargetOptions {
            endian: Endian::Little,
            c_int_width: "32".to_string(),
            os: "none".to_string(),
            env: String::new(),
            vendor: String::new(),
            linker_flavor: LinkerFlavor::Gcc,

            executables: true,
            cpu: "esp8266".to_string(),
            linker: Some("xtensa-lx106-elf-gcc".to_string()),

            max_atomic_width: Some(32),

            // Because these devices have very little resources having an
            // unwinder is too onerous so we default to "abort" because the
            // "unwind" strategy is very rare.
            panic_strategy: PanicStrategy::Abort,

            // Similarly, one almost always never wants to use relocatable
            // code because of the extra costs it involves.
            relocation_model: RelocModel::Static,

            emit_debug_gdb_scripts: false,

            unsupported_abis: super::xtensa_base::unsupported_abis(),
            ..Default::default()
        },
    }
}