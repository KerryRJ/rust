use crate::abi::Size;
use crate::spec::Target;
use rustc_data_structures::fx::{FxHashMap, FxHashSet};
use rustc_macros::HashStable_Generic;
use rustc_span::Symbol;
use std::fmt;
use std::str::FromStr;

macro_rules! def_reg_class {
    ($arch:ident $arch_regclass:ident {
        $(
            $class:ident,
        )*
    }) => {
        #[derive(Copy, Clone, Encodable, Decodable, Debug, Eq, PartialEq, PartialOrd, Hash, HashStable_Generic)]
        #[allow(non_camel_case_types)]
        pub enum $arch_regclass {
            $($class,)*
        }

        impl $arch_regclass {
            pub fn name(self) -> rustc_span::Symbol {
                match self {
                    $(Self::$class => rustc_span::symbol::sym::$class,)*
                }
            }

            pub fn parse(_arch: super::InlineAsmArch, name: rustc_span::Symbol) -> Result<Self, &'static str> {
                match name {
                    $(
                        rustc_span::sym::$class => Ok(Self::$class),
                    )*
                    _ => Err("unknown register class"),
                }
            }
        }

        pub(super) fn regclass_map() -> rustc_data_structures::fx::FxHashMap<
            super::InlineAsmRegClass,
            rustc_data_structures::fx::FxHashSet<super::InlineAsmReg>,
        > {
            use rustc_data_structures::fx::{FxHashMap, FxHashSet};
            use super::InlineAsmRegClass;
            let mut map = FxHashMap::default();
            $(
                map.insert(InlineAsmRegClass::$arch($arch_regclass::$class), FxHashSet::default());
            )*
            map
        }
    }
}

macro_rules! def_regs {
    ($arch:ident $arch_reg:ident $arch_regclass:ident {
        $(
            $reg:ident: $class:ident $(, $extra_class:ident)* = [$reg_name:literal $(, $alias:literal)*] $(% $filter:ident)?,
        )*
        $(
            #error = [$($bad_reg:literal),+] => $error:literal,
        )*
    }) => {
        #[allow(unreachable_code)]
        #[derive(Copy, Clone, Encodable, Decodable, Debug, Eq, PartialEq, PartialOrd, Hash, HashStable_Generic)]
        #[allow(non_camel_case_types)]
        pub enum $arch_reg {
            $($reg,)*
        }

        impl $arch_reg {
            pub fn name(self) -> &'static str {
                match self {
                    $(Self::$reg => $reg_name,)*
                }
            }

            pub fn reg_class(self) -> $arch_regclass {
                match self {
                    $(Self::$reg => $arch_regclass::$class,)*
                }
            }

            pub fn parse(
                _arch: super::InlineAsmArch,
                mut _has_feature: impl FnMut(&str) -> bool,
                _target: &crate::spec::Target,
                name: &str,
            ) -> Result<Self, &'static str> {
                match name {
                    $(
                        $($alias)|* | $reg_name => {
                            $($filter(_arch, &mut _has_feature, _target)?;)?
                            Ok(Self::$reg)
                        }
                    )*
                    $(
                        $($bad_reg)|* => Err($error),
                    )*
                    _ => Err("unknown register"),
                }
            }
        }

        pub(super) fn fill_reg_map(
            _arch: super::InlineAsmArch,
            mut _has_feature: impl FnMut(&str) -> bool,
            _target: &crate::spec::Target,
            _map: &mut rustc_data_structures::fx::FxHashMap<
                super::InlineAsmRegClass,
                rustc_data_structures::fx::FxHashSet<super::InlineAsmReg>,
            >,
        ) {
            #[allow(unused_imports)]
            use super::{InlineAsmReg, InlineAsmRegClass};
            $(
                if $($filter(_arch, &mut _has_feature, _target).is_ok() &&)? true {
                    if let Some(set) = _map.get_mut(&InlineAsmRegClass::$arch($arch_regclass::$class)) {
                        set.insert(InlineAsmReg::$arch($arch_reg::$reg));
                    }
                    $(
                        if let Some(set) = _map.get_mut(&InlineAsmRegClass::$arch($arch_regclass::$extra_class)) {
                            set.insert(InlineAsmReg::$arch($arch_reg::$reg));
                        }
                    )*
                }
            )*
        }
    }
}

macro_rules! types {
    (
        $(_ : $($ty:expr),+;)?
        $($feature:literal: $($ty2:expr),+;)*
    ) => {
        {
            use super::InlineAsmType::*;
            &[
                $($(
                    ($ty, None),
                )*)?
                $($(
                    ($ty2, Some($feature)),
                )*)*
            ]
        }
    };
}

mod aarch64;
mod arm;
mod bpf;
mod hexagon;
mod mips;
mod nvptx;
mod powerpc;
mod riscv;
mod spirv;
mod wasm;
mod x86;
mod xtensa;

pub use aarch64::{AArch64InlineAsmReg, AArch64InlineAsmRegClass};
pub use arm::{ArmInlineAsmReg, ArmInlineAsmRegClass};
pub use bpf::{BpfInlineAsmReg, BpfInlineAsmRegClass};
pub use hexagon::{HexagonInlineAsmReg, HexagonInlineAsmRegClass};
pub use mips::{MipsInlineAsmReg, MipsInlineAsmRegClass};
pub use nvptx::{NvptxInlineAsmReg, NvptxInlineAsmRegClass};
pub use powerpc::{PowerPCInlineAsmReg, PowerPCInlineAsmRegClass};
pub use riscv::{RiscVInlineAsmReg, RiscVInlineAsmRegClass};
pub use spirv::{SpirVInlineAsmReg, SpirVInlineAsmRegClass};
pub use wasm::{WasmInlineAsmReg, WasmInlineAsmRegClass};
pub use xtensa::{XtensaInlineAsmReg, XtensaInlineAsmRegClass};
pub use x86::{X86InlineAsmReg, X86InlineAsmRegClass};

#[derive(Copy, Clone, Encodable, Decodable, Debug, Eq, PartialEq, Hash)]
pub enum InlineAsmArch {
    X86,
    X86_64,
    Arm,
    AArch64,
    RiscV32,
    RiscV64,
    Nvptx64,
    Hexagon,
    Mips,
    Mips64,
    PowerPC,
    PowerPC64,
    SpirV,
    Wasm32,
    Xtensa,
    Bpf,
}

impl FromStr for InlineAsmArch {
    type Err = ();

    fn from_str(s: &str) -> Result<InlineAsmArch, ()> {
        match s {
            "x86" => Ok(Self::X86),
            "x86_64" => Ok(Self::X86_64),
            "arm" => Ok(Self::Arm),
            "aarch64" => Ok(Self::AArch64),
            "riscv32" => Ok(Self::RiscV32),
            "riscv64" => Ok(Self::RiscV64),
            "nvptx64" => Ok(Self::Nvptx64),
            "powerpc" => Ok(Self::PowerPC),
            "powerpc64" => Ok(Self::PowerPC64),
            "hexagon" => Ok(Self::Hexagon),
            "mips" => Ok(Self::Mips),
            "mips64" => Ok(Self::Mips64),
            "spirv" => Ok(Self::SpirV),
            "wasm32" => Ok(Self::Wasm32),
            "xtensa" => Ok(Self::Xtensa),
            "bpf" => Ok(Self::Bpf),
            _ => Err(()),
        }
    }
}

#[derive(
    Copy,
    Clone,
    Encodable,
    Decodable,
    Debug,
    Eq,
    PartialEq,
    PartialOrd,
    Hash,
    HashStable_Generic
)]
pub enum InlineAsmReg {
    X86(X86InlineAsmReg),
    Arm(ArmInlineAsmReg),
    AArch64(AArch64InlineAsmReg),
    RiscV(RiscVInlineAsmReg),
    Nvptx(NvptxInlineAsmReg),
    PowerPC(PowerPCInlineAsmReg),
    Hexagon(HexagonInlineAsmReg),
    Mips(MipsInlineAsmReg),
    SpirV(SpirVInlineAsmReg),
    Wasm(WasmInlineAsmReg),
    Xtensa(XtensaInlineAsmReg),
    Bpf(BpfInlineAsmReg),
    // Placeholder for invalid register constraints for the current target
    Err,
}

impl InlineAsmReg {
    pub fn name(self) -> &'static str {
        match self {
            Self::X86(r) => r.name(),
            Self::Arm(r) => r.name(),
            Self::AArch64(r) => r.name(),
            Self::RiscV(r) => r.name(),
            Self::PowerPC(r) => r.name(),
            Self::Hexagon(r) => r.name(),
            Self::Mips(r) => r.name(),
            Self::Xtensa(r) => r.name(),
            Self::Bpf(r) => r.name(),
            Self::Err => "<reg>",
        }
    }

    pub fn reg_class(self) -> InlineAsmRegClass {
        match self {
            Self::X86(r) => InlineAsmRegClass::X86(r.reg_class()),
            Self::Arm(r) => InlineAsmRegClass::Arm(r.reg_class()),
            Self::AArch64(r) => InlineAsmRegClass::AArch64(r.reg_class()),
            Self::RiscV(r) => InlineAsmRegClass::RiscV(r.reg_class()),
            Self::PowerPC(r) => InlineAsmRegClass::PowerPC(r.reg_class()),
            Self::Hexagon(r) => InlineAsmRegClass::Hexagon(r.reg_class()),
            Self::Mips(r) => InlineAsmRegClass::Mips(r.reg_class()),
            Self::Xtensa(r) => InlineAsmRegClass::Xtensa(r.reg_class()),
            Self::Bpf(r) => InlineAsmRegClass::Bpf(r.reg_class()),
            Self::Err => InlineAsmRegClass::Err,
        }
    }

    pub fn parse(
        arch: InlineAsmArch,
        has_feature: impl FnMut(&str) -> bool,
        target: &Target,
        name: Symbol,
    ) -> Result<Self, &'static str> {
        // FIXME: use direct symbol comparison for register names
        // Use `Symbol::as_str` instead of `Symbol::with` here because `has_feature` may access `Symbol`.
        let name = name.as_str();
        Ok(match arch {
            InlineAsmArch::X86 | InlineAsmArch::X86_64 => {
                Self::X86(X86InlineAsmReg::parse(arch, has_feature, target, &name)?)
            }
            InlineAsmArch::Arm => {
                Self::Arm(ArmInlineAsmReg::parse(arch, has_feature, target, &name)?)
            }
            InlineAsmArch::AArch64 => {
                Self::AArch64(AArch64InlineAsmReg::parse(arch, has_feature, target, &name)?)
            }
            InlineAsmArch::RiscV32 | InlineAsmArch::RiscV64 => {
                Self::RiscV(RiscVInlineAsmReg::parse(arch, has_feature, target, &name)?)
            }
            InlineAsmArch::Nvptx64 => {
                Self::Nvptx(NvptxInlineAsmReg::parse(arch, has_feature, target, &name)?)
            }
            InlineAsmArch::PowerPC | InlineAsmArch::PowerPC64 => {
                Self::PowerPC(PowerPCInlineAsmReg::parse(arch, has_feature, target, &name)?)
            }
            InlineAsmArch::Hexagon => {
                Self::Hexagon(HexagonInlineAsmReg::parse(arch, has_feature, target, &name)?)
            }
            InlineAsmArch::Mips | InlineAsmArch::Mips64 => {
                Self::Mips(MipsInlineAsmReg::parse(arch, has_feature, target, &name)?)
            }
            InlineAsmArch::SpirV => {
                Self::SpirV(SpirVInlineAsmReg::parse(arch, has_feature, target, &name)?)
            }
            InlineAsmArch::Wasm32 => {
                Self::Wasm(WasmInlineAsmReg::parse(arch, has_feature, target, &name)?)
            }
            InlineAsmArch::Xtensa => {
                Self::Xtensa(XtensaInlineAsmReg::parse(arch, has_feature, target, &name)?)
            }
            InlineAsmArch::Bpf => {
                Self::Bpf(BpfInlineAsmReg::parse(arch, has_feature, target, &name)?)
            }
        })
    }

    // NOTE: This function isn't used at the moment, but is needed to support
    // falling back to an external assembler.
    pub fn emit(
        self,
        out: &mut dyn fmt::Write,
        arch: InlineAsmArch,
        modifier: Option<char>,
    ) -> fmt::Result {
        match self {
            Self::X86(r) => r.emit(out, arch, modifier),
            Self::Arm(r) => r.emit(out, arch, modifier),
            Self::AArch64(r) => r.emit(out, arch, modifier),
            Self::RiscV(r) => r.emit(out, arch, modifier),
            Self::PowerPC(r) => r.emit(out, arch, modifier),
            Self::Hexagon(r) => r.emit(out, arch, modifier),
            Self::Mips(r) => r.emit(out, arch, modifier),
            Self::Xtensa(r) => r.emit(out, arch, modifier),
            Self::Bpf(r) => r.emit(out, arch, modifier),
            Self::Err => unreachable!("Use of InlineAsmReg::Err"),
        }
    }

    pub fn overlapping_regs(self, mut cb: impl FnMut(InlineAsmReg)) {
        match self {
            Self::X86(r) => r.overlapping_regs(|r| cb(Self::X86(r))),
            Self::Arm(r) => r.overlapping_regs(|r| cb(Self::Arm(r))),
            Self::AArch64(_) => cb(self),
            Self::RiscV(_) => cb(self),
            Self::PowerPC(_) => cb(self),
            Self::Hexagon(r) => r.overlapping_regs(|r| cb(Self::Hexagon(r))),
            Self::Mips(_) => cb(self),
            Self::Xtensa(_) => cb(self),
            Self::Bpf(r) => r.overlapping_regs(|r| cb(Self::Bpf(r))),
            Self::Err => unreachable!("Use of InlineAsmReg::Err"),
        }
    }
}

#[derive(
    Copy,
    Clone,
    Encodable,
    Decodable,
    Debug,
    Eq,
    PartialEq,
    PartialOrd,
    Hash,
    HashStable_Generic
)]
pub enum InlineAsmRegClass {
    X86(X86InlineAsmRegClass),
    Arm(ArmInlineAsmRegClass),
    AArch64(AArch64InlineAsmRegClass),
    RiscV(RiscVInlineAsmRegClass),
    Nvptx(NvptxInlineAsmRegClass),
    PowerPC(PowerPCInlineAsmRegClass),
    Hexagon(HexagonInlineAsmRegClass),
    Mips(MipsInlineAsmRegClass),
    SpirV(SpirVInlineAsmRegClass),
    Wasm(WasmInlineAsmRegClass),
    Xtensa(XtensaInlineAsmRegClass),
    Bpf(BpfInlineAsmRegClass),
    // Placeholder for invalid register constraints for the current target
    Err,
}

impl InlineAsmRegClass {
    pub fn name(self) -> Symbol {
        match self {
            Self::X86(r) => r.name(),
            Self::Arm(r) => r.name(),
            Self::AArch64(r) => r.name(),
            Self::RiscV(r) => r.name(),
            Self::Nvptx(r) => r.name(),
            Self::PowerPC(r) => r.name(),
            Self::Hexagon(r) => r.name(),
            Self::Mips(r) => r.name(),
            Self::SpirV(r) => r.name(),
            Self::Wasm(r) => r.name(),
            Self::Xtensa(r) => r.name(),
            Self::Bpf(r) => r.name(),
            Self::Err => rustc_span::symbol::sym::reg,
        }
    }

    /// Returns a suggested register class to use for this type. This is called
    /// after type checking via `supported_types` fails to give a better error
    /// message to the user.
    pub fn suggest_class(self, arch: InlineAsmArch, ty: InlineAsmType) -> Option<Self> {
        match self {
            Self::X86(r) => r.suggest_class(arch, ty).map(InlineAsmRegClass::X86),
            Self::Arm(r) => r.suggest_class(arch, ty).map(InlineAsmRegClass::Arm),
            Self::AArch64(r) => r.suggest_class(arch, ty).map(InlineAsmRegClass::AArch64),
            Self::RiscV(r) => r.suggest_class(arch, ty).map(InlineAsmRegClass::RiscV),
            Self::Nvptx(r) => r.suggest_class(arch, ty).map(InlineAsmRegClass::Nvptx),
            Self::PowerPC(r) => r.suggest_class(arch, ty).map(InlineAsmRegClass::PowerPC),
            Self::Hexagon(r) => r.suggest_class(arch, ty).map(InlineAsmRegClass::Hexagon),
            Self::Mips(r) => r.suggest_class(arch, ty).map(InlineAsmRegClass::Mips),
            Self::SpirV(r) => r.suggest_class(arch, ty).map(InlineAsmRegClass::SpirV),
            Self::Wasm(r) => r.suggest_class(arch, ty).map(InlineAsmRegClass::Wasm),
            Self::Xtensa(r) => r.suggest_class(arch, ty).map(InlineAsmRegClass::Xtensa),
            Self::Bpf(r) => r.suggest_class(arch, ty).map(InlineAsmRegClass::Bpf),
            Self::Err => unreachable!("Use of InlineAsmRegClass::Err"),
        }
    }

    /// Returns a suggested template modifier to use for this type and an
    /// example of a  register named formatted with it.
    ///
    /// Such suggestions are useful if a type smaller than the full register
    /// size is used and a modifier can be used to point to the subregister of
    /// the correct size.
    pub fn suggest_modifier(
        self,
        arch: InlineAsmArch,
        ty: InlineAsmType,
    ) -> Option<(char, &'static str)> {
        match self {
            Self::X86(r) => r.suggest_modifier(arch, ty),
            Self::Arm(r) => r.suggest_modifier(arch, ty),
            Self::AArch64(r) => r.suggest_modifier(arch, ty),
            Self::RiscV(r) => r.suggest_modifier(arch, ty),
            Self::Nvptx(r) => r.suggest_modifier(arch, ty),
            Self::PowerPC(r) => r.suggest_modifier(arch, ty),
            Self::Hexagon(r) => r.suggest_modifier(arch, ty),
            Self::Mips(r) => r.suggest_modifier(arch, ty),
            Self::SpirV(r) => r.suggest_modifier(arch, ty),
            Self::Wasm(r) => r.suggest_modifier(arch, ty),
            Self::Xtensa(r) => r.suggest_modifier(arch, ty),
            Self::Bpf(r) => r.suggest_modifier(arch, ty),
            Self::Err => unreachable!("Use of InlineAsmRegClass::Err"),
        }
    }

    /// Returns the default modifier for this register and an example of a
    /// register named formatted with it.
    ///
    /// This is only needed when the register class can suggest a modifier, so
    /// that the user can be shown how to get the default behavior without a
    /// warning.
    pub fn default_modifier(self, arch: InlineAsmArch) -> Option<(char, &'static str)> {
        match self {
            Self::X86(r) => r.default_modifier(arch),
            Self::Arm(r) => r.default_modifier(arch),
            Self::AArch64(r) => r.default_modifier(arch),
            Self::RiscV(r) => r.default_modifier(arch),
            Self::Nvptx(r) => r.default_modifier(arch),
            Self::PowerPC(r) => r.default_modifier(arch),
            Self::Hexagon(r) => r.default_modifier(arch),
            Self::Mips(r) => r.default_modifier(arch),
            Self::SpirV(r) => r.default_modifier(arch),
            Self::Wasm(r) => r.default_modifier(arch),
            Self::Xtensa(r) => r.default_modifier(arch),
            Self::Bpf(r) => r.default_modifier(arch),
            Self::Err => unreachable!("Use of InlineAsmRegClass::Err"),
        }
    }

    /// Returns a list of supported types for this register class, each with a
    /// options target feature required to use this type.
    pub fn supported_types(
        self,
        arch: InlineAsmArch,
    ) -> &'static [(InlineAsmType, Option<&'static str>)] {
        match self {
            Self::X86(r) => r.supported_types(arch),
            Self::Arm(r) => r.supported_types(arch),
            Self::AArch64(r) => r.supported_types(arch),
            Self::RiscV(r) => r.supported_types(arch),
            Self::Nvptx(r) => r.supported_types(arch),
            Self::PowerPC(r) => r.supported_types(arch),
            Self::Hexagon(r) => r.supported_types(arch),
            Self::Mips(r) => r.supported_types(arch),
            Self::SpirV(r) => r.supported_types(arch),
            Self::Wasm(r) => r.supported_types(arch),
            Self::Xtensa(r) => r.supported_types(arch),
            Self::Bpf(r) => r.supported_types(arch),
            Self::Err => unreachable!("Use of InlineAsmRegClass::Err"),
        }
    }

    pub fn parse(arch: InlineAsmArch, name: Symbol) -> Result<Self, &'static str> {
        Ok(match arch {
            InlineAsmArch::X86 | InlineAsmArch::X86_64 => {
                Self::X86(X86InlineAsmRegClass::parse(arch, name)?)
            }
            InlineAsmArch::Arm => Self::Arm(ArmInlineAsmRegClass::parse(arch, name)?),
            InlineAsmArch::AArch64 => Self::AArch64(AArch64InlineAsmRegClass::parse(arch, name)?),
            InlineAsmArch::RiscV32 | InlineAsmArch::RiscV64 => {
                Self::RiscV(RiscVInlineAsmRegClass::parse(arch, name)?)
            }
            InlineAsmArch::Nvptx64 => Self::Nvptx(NvptxInlineAsmRegClass::parse(arch, name)?),
            InlineAsmArch::PowerPC | InlineAsmArch::PowerPC64 => {
                Self::PowerPC(PowerPCInlineAsmRegClass::parse(arch, name)?)
            }
            InlineAsmArch::Hexagon => Self::Hexagon(HexagonInlineAsmRegClass::parse(arch, name)?),
            InlineAsmArch::Mips | InlineAsmArch::Mips64 => {
                Self::Mips(MipsInlineAsmRegClass::parse(arch, name)?)
            }
            InlineAsmArch::SpirV => Self::SpirV(SpirVInlineAsmRegClass::parse(arch, name)?),
            InlineAsmArch::Wasm32 => Self::Wasm(WasmInlineAsmRegClass::parse(arch, name)?),
            InlineAsmArch::Xtensa => Self::Xtensa(XtensaInlineAsmRegClass::parse(arch, name)?),
            InlineAsmArch::Bpf => Self::Bpf(BpfInlineAsmRegClass::parse(arch, name)?),
        })
    }

    /// Returns the list of template modifiers that can be used with this
    /// register class.
    pub fn valid_modifiers(self, arch: InlineAsmArch) -> &'static [char] {
        match self {
            Self::X86(r) => r.valid_modifiers(arch),
            Self::Arm(r) => r.valid_modifiers(arch),
            Self::AArch64(r) => r.valid_modifiers(arch),
            Self::RiscV(r) => r.valid_modifiers(arch),
            Self::Nvptx(r) => r.valid_modifiers(arch),
            Self::PowerPC(r) => r.valid_modifiers(arch),
            Self::Hexagon(r) => r.valid_modifiers(arch),
            Self::Mips(r) => r.valid_modifiers(arch),
            Self::SpirV(r) => r.valid_modifiers(arch),
            Self::Wasm(r) => r.valid_modifiers(arch),
            Self::Xtensa(r) => r.valid_modifiers(arch),
            Self::Bpf(r) => r.valid_modifiers(arch),
            Self::Err => unreachable!("Use of InlineAsmRegClass::Err"),
        }
    }
}

#[derive(
    Copy,
    Clone,
    Encodable,
    Decodable,
    Debug,
    Eq,
    PartialEq,
    PartialOrd,
    Hash,
    HashStable_Generic
)]
pub enum InlineAsmRegOrRegClass {
    Reg(InlineAsmReg),
    RegClass(InlineAsmRegClass),
}

impl InlineAsmRegOrRegClass {
    pub fn reg_class(self) -> InlineAsmRegClass {
        match self {
            Self::Reg(r) => r.reg_class(),
            Self::RegClass(r) => r,
        }
    }
}

impl fmt::Display for InlineAsmRegOrRegClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reg(r) => write!(f, "\"{}\"", r.name()),
            Self::RegClass(r) => write!(f, "{}", r.name()),
        }
    }
}

/// Set of types which can be used with a particular register class.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum InlineAsmType {
    I1,
    I8,
    I16,
    I32,
    I64,
    I128,
    F32,
    F64,
    VecI8(u64),
    VecI16(u64),
    VecI32(u64),
    VecI64(u64),
    VecI128(u64),
    VecF32(u64),
    VecF64(u64),
}

impl InlineAsmType {
    pub fn is_integer(self) -> bool {
        matches!(self, Self::I8 | Self::I16 | Self::I32 | Self::I64 | Self::I128)
    }

    pub fn size(self) -> Size {
        Size::from_bytes(match self {
            Self::I1 => return Size::from_bits(1),
            Self::I8 => 1,
            Self::I16 => 2,
            Self::I32 => 4,
            Self::I64 => 8,
            Self::I128 => 16,
            Self::F32 => 4,
            Self::F64 => 8,
            Self::VecI8(n) => n * 1,
            Self::VecI16(n) => n * 2,
            Self::VecI32(n) => n * 4,
            Self::VecI64(n) => n * 8,
            Self::VecI128(n) => n * 16,
            Self::VecF32(n) => n * 4,
            Self::VecF64(n) => n * 8,
        })
    }
}

impl fmt::Display for InlineAsmType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::I1 => f.write_str("i1"),
            Self::I8 => f.write_str("i8"),
            Self::I16 => f.write_str("i16"),
            Self::I32 => f.write_str("i32"),
            Self::I64 => f.write_str("i64"),
            Self::I128 => f.write_str("i128"),
            Self::F32 => f.write_str("f32"),
            Self::F64 => f.write_str("f64"),
            Self::VecI8(n) => write!(f, "i8x{}", n),
            Self::VecI16(n) => write!(f, "i16x{}", n),
            Self::VecI32(n) => write!(f, "i32x{}", n),
            Self::VecI64(n) => write!(f, "i64x{}", n),
            Self::VecI128(n) => write!(f, "i128x{}", n),
            Self::VecF32(n) => write!(f, "f32x{}", n),
            Self::VecF64(n) => write!(f, "f64x{}", n),
        }
    }
}

/// Returns the full set of allocatable registers for a given architecture.
///
/// The registers are structured as a map containing the set of allocatable
/// registers in each register class. A particular register may be allocatable
/// from multiple register classes, in which case it will appear multiple times
/// in the map.
// NOTE: This function isn't used at the moment, but is needed to support
// falling back to an external assembler.
pub fn allocatable_registers(
    arch: InlineAsmArch,
    has_feature: impl FnMut(&str) -> bool,
    target: &crate::spec::Target,
) -> FxHashMap<InlineAsmRegClass, FxHashSet<InlineAsmReg>> {
    match arch {
        InlineAsmArch::X86 | InlineAsmArch::X86_64 => {
            let mut map = x86::regclass_map();
            x86::fill_reg_map(arch, has_feature, target, &mut map);
            map
        }
        InlineAsmArch::Arm => {
            let mut map = arm::regclass_map();
            arm::fill_reg_map(arch, has_feature, target, &mut map);
            map
        }
        InlineAsmArch::AArch64 => {
            let mut map = aarch64::regclass_map();
            aarch64::fill_reg_map(arch, has_feature, target, &mut map);
            map
        }
        InlineAsmArch::RiscV32 | InlineAsmArch::RiscV64 => {
            let mut map = riscv::regclass_map();
            riscv::fill_reg_map(arch, has_feature, target, &mut map);
            map
        }
        InlineAsmArch::Nvptx64 => {
            let mut map = nvptx::regclass_map();
            nvptx::fill_reg_map(arch, has_feature, target, &mut map);
            map
        }
        InlineAsmArch::PowerPC | InlineAsmArch::PowerPC64 => {
            let mut map = powerpc::regclass_map();
            powerpc::fill_reg_map(arch, has_feature, target, &mut map);
            map
        }
        InlineAsmArch::Hexagon => {
            let mut map = hexagon::regclass_map();
            hexagon::fill_reg_map(arch, has_feature, target, &mut map);
            map
        }
        InlineAsmArch::Mips | InlineAsmArch::Mips64 => {
            let mut map = mips::regclass_map();
            mips::fill_reg_map(arch, has_feature, target, &mut map);
            map
        }
        InlineAsmArch::SpirV => {
            let mut map = spirv::regclass_map();
            spirv::fill_reg_map(arch, has_feature, target, &mut map);
            map
        }
        InlineAsmArch::Wasm32 => {
            let mut map = wasm::regclass_map();
            wasm::fill_reg_map(arch, has_feature, target, &mut map);
            map
        }
        InlineAsmArch::Xtensa => {
            let mut map = xtensa::regclass_map();
            xtensa::fill_reg_map(arch, has_feature, target, &mut map);
            map
        }
        InlineAsmArch::Bpf => {
            let mut map = bpf::regclass_map();
            bpf::fill_reg_map(arch, has_feature, target, &mut map);
            map
        }
    }
}
