use crate::*;
use command_ext::{Cargo, CommandExt};
use serde_derive::Deserialize;
use std::{ffi::OsStr, fs::File, io::Write, path::PathBuf, fs::{canonicalize, read_dir}};

#[derive(Deserialize)]
struct Ch2 {
    ch2: Cases,
}

#[derive(Deserialize)]
struct Ch3 {
    ch3: Cases,
}

#[derive(Deserialize)]
struct Ch4 {
    ch4: Cases,
}

#[derive(Deserialize, Default)]
struct Cases {
    base: Option<u64>,
    step: Option<u64>,
    cases: Option<Vec<String>>,
}

struct CasesInfo {
    base: u64,
    step: u64,
    bins: Vec<PathBuf>,
}

impl Cases {
    fn build(self, release: bool) -> CasesInfo {
        if let Some(names) = self.cases {
            let base = self.base.unwrap_or(0);
            let step = self.step.filter(|_| self.base.is_some()).unwrap_or(0);
            let cases = names
                .into_iter()
                .enumerate()
                .map(|(i, name)| build_one(name, release, base + i as u64 * step))
                .collect();
            CasesInfo {
                base,
                step,
                bins: cases,
            }
        } else {
            CasesInfo {
                base: 0,
                step: 0,
                bins: vec![],
            }
        }
    }
}

fn build_one(name: impl AsRef<OsStr>, release: bool, base_address: u64) -> PathBuf {
    let name = name.as_ref();
    let binary = base_address != 0;
    if binary {
        println!("build {name:?} at {base_address:#x}");
    }
    Cargo::build()
        .package("user_lib")
        .target(TARGET_ARCH)
        .arg("--bin")
        .arg(name)
        .conditional(release, |cargo| {
            cargo.release();
        })
        .conditional(binary, |cargo| {
            cargo.env("BASE_ADDRESS", base_address.to_string());
        })
        .invoke();
    let elf = TARGET
        .join(if release { "release" } else { "debug" })
        .join(name);
    objcopy(elf, binary)
}

pub fn build_for(ch: u8, release: bool) {
    let cfg = std::fs::read_to_string(PROJECT.join("user/cases.toml")).unwrap();
    let CasesInfo { base, step, bins } = match ch {
        2 => toml::from_str::<Ch2>(&cfg).map(|ch| ch.ch2),
        3 => toml::from_str::<Ch3>(&cfg).map(|ch| ch.ch3),
        4 => toml::from_str::<Ch4>(&cfg).map(|ch| ch.ch4),
        _ => unreachable!(),
    }
    .unwrap_or_default()
    .build(release);
    if bins.is_empty() {
        return;
    }
    let asm = TARGET
        .join(if release { "release" } else { "debug" })
        .join("app.asm");
    let mut ld = File::create(asm).unwrap();

    // base on lab1-os3 how to create a app_link.S to like all information

    writeln!(
        ld,
        "\
    .global _num_app
    .section .data
    .align 3
_num_app:
    .quad {}",
        bins.len(),
    ).unwrap();
    
    // statement two all quad which could help us to divide each sections
    (0..bins.len()).for_each(|i| {
        writeln!(
            ld,
            "\
                .quad app_{i}_start"
        ).unwrap();
    });

    writeln!(
        ld,
        "\
        .quad app_{}_end",
        bins.len() - 1
    ).unwrap();

    bins.iter().enumerate().for_each(|(i, path)|{
        writeln!(
            ld,
            "\
        .section .data
        .global app_{i}_start
        .global app_{i}_end
    app_{i}_start:
        .incbin {path:?}
    app_{i}_end:"
            ).unwrap();
    });

    // bins.iter().enumerate().for_each(|(i, path)| {
    //     writeln!(
    //         ld,
    //         "
    //     .app_{i}_start:
    //         .incbin {path:?}
    //     .app_{i}_end:",
    //     ).unwrap();
    // });

    // println!("finish linked!");

//     writeln!(
//         f,
//         r#"
//     .align 3
//     .section .data
//     .global _num_app
// _num_app:
//     .quad {}"#,
//         apps.len()
//     )?;

//     for i in 0..apps.len() {
//         writeln!(f, r#"    .quad app_{}_start"#, i)?;
//     }
//     writeln!(f, r#"    .quad app_{}_end"#, apps.len() - 1)?;

//     for (idx, app) in apps.iter().enumerate() {
//         println!("app_{}: {}", idx, app);
//         writeln!(
//             f,
//             r#"
//     .section .data
//     .global app_{0}_start
//     .global app_{0}_end
// app_{0}_start:
//     .incbin "{2}{1}.bin"
// app_{0}_end:"#,
//             idx, app, TARGET_PATH
//         )?;
//     }    
//     writeln!(
//         ld,
//         "\
//     .global apps
//     .section .data
//     .align 3
// apps:
//     .quad {base:#x}
//     .quad {step:#x}
//     .quad {}",
//         bins.len(),
//     )
//     .unwrap();

//     (0..bins.len()).for_each(|i| {
//         writeln!(
//             ld,
//             "\
//     .quad app_{i}_start"
//         )
//         .unwrap()
//     });

//     writeln!(
//         ld,
//         "\
//     .quad app_{}_end",
//         bins.len() - 1
//     )
//     .unwrap();

//     bins.iter().enumerate().for_each(|(i, path)| {
//         writeln!(
//             ld,
//             "
// app_{i}_start:
//     .incbin {path:?}
// app_{i}_end:",
//         )
//         .unwrap();
//     });
}
