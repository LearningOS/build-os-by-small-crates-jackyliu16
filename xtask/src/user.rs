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

    let trap = TARGET
        .join(if release { "release" } else { "debug" })
        .join("trap.S");
    let mut trap_s = File::create(trap).unwrap();
    writeln!(
        trap_s,
        r#"
        .altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm
    .section .text
    .globl __alltraps
    .globl __restore
    .align 2
# save trap context and jump into trap_handler
__alltraps:
    # before csrrw: sscratch->kernel stack, sp->user kernel
    csrrw sp, sscratch, sp # sscratch->sp, sp->sscratch
    # now sp->kernel stack, sscratch->user stack (swap value)
    # allocate a TrapContext on kernel stack
    # sp grows from tail to head.
    addi sp, sp, -34*8
    # save general-purpose registers
    sd x1, 1*8(sp)
    # skip sp(x2), we will save it later
    sd x3, 3*8(sp)
    # skip tp(x4), application does not use it
    # save x5~x31
    .set n, 5
    .rept 27            # repeat 27 times
        SAVE_GP %n      # sd xn n*8(sp) 
        .set n, n+1
    .endr
    # we can use t0/t1/t2 freely, because they were saved on kernel stack
    csrr t0, sstatus    # sstatus stores original running level
    csrr t1, sepc       # sepc stores last i-addr
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    # read user stack from sscratch and save it on the kernel stack
    csrr t2, sscratch
    sd t2, 2*8(sp)
    # set input argument of trap_handler(cx: &mut TrapContext)
    mv a0, sp
    call trap_handler

__restore:
    # case1: start running app by __restore
    # case2: back to U after handling trap
    mv sp, a0
    # now sp->kernel stack(after allocated), sscratch->user stack
    # restore sstatus/sepc
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    ld t2, 2*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    csrw sscratch, t2
    # restore general-purpuse registers except sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # release TrapContext on kernel stack
    addi sp, sp, 34*8
    # now sp->kernel stack, sscratch->user stack
    csrrw sp, sscratch, sp
    sret
    "#).unwrap();

    let switch = TARGET
        .join(if release { "release" } else { "debug" })
        .join("switch.S");
    
    let mut switch_s = File::create(switch).unwrap();
    
    writeln!(
        switch_s,
        r#".altmacro
.macro SAVE_SN n
    sd s\n, (\n+2)*8(a0)
.endm
.macro LOAD_SN n
    ld s\n, (\n+2)*8(a1)
.endm
    .section .text
    .globl __switch
__switch:
    # __switch(
    #     current_task_cx_ptr: *mut TaskContext,
    #     next_task_cx_ptr: *const TaskContext
    # )
    # save kernel stack of current task
    sd sp, 8(a0)
    # save ra & s0~s11 of current execution
    sd ra, 0(a0)
    .set n, 0
    .rept 12
        SAVE_SN %n
        .set n, n + 1
    .endr
    # restore ra & s0~s11 of next execution
    ld ra, 0(a1)
    .set n, 0
    .rept 12
        LOAD_SN %n
        .set n, n + 1
    .endr
    # restore kernel stack of next task
    ld sp, 8(a1)
    ret
    "#).unwrap();
}



// 应该是从后面章节复制下来的，由于内部某些定义不一样，因此没有办法运行
// writeln!(
//         trap_s,
//         r#".altmacro
// .macro SAVE_GP n
//     sd x\n, \n*8(sp)
// .endm
// .macro LOAD_GP n
//     ld x\n, \n*8(sp)
// .endm
//     .section .text.trampoline
//     .globl __alltraps
//     .globl __restore
//     .align 2
// # save trap context 
// __alltraps:
//     csrrw sp, sscratch, sp
//     # now sp->*TrapContext in user space, sscratch->user stack
//     # save other general purpose registers
//     sd x1, 1*8(sp)
//     # skip sp(x2), we will save it later
//     sd x3, 3*8(sp)
//     # skip tp(x4), application does not use it
//     # save x5~x31
//     .set n, 5
//     .rept 27
//         SAVE_GP %n
//         .set n, n+1
//     .endr
//     # we can use t0/t1/t2 freely, because they have been saved in TrapContext
//     csrr t0, sstatus
//     csrr t1, sepc
//     sd t0, 32*8(sp)
//     sd t1, 33*8(sp)
//     # read user stack from sscratch and save it in TrapContext
//     csrr t2, sscratch
//     sd t2, 2*8(sp)
//     # load kernel_satp into t0
//     ld t0, 34*8(sp)
//     # load trap_handler into t1
//     ld t1, 36*8(sp)
//     # move to kernel_sp
//     ld sp, 35*8(sp)
//     # switch to kernel space
//     csrw satp, t0
//     sfence.vma
//     # jump to trap_handler
//     jr t1
// # 
// __restore:
//     # a0: *TrapContext in user space(Constant); a1: user space token
//     # switch to user space
//     csrw satp, a1
//     sfence.vma
//     csrw sscratch, a0
//     mv sp, a0
//     # now sp points to TrapContext in user space, start restoring based on it
//     # restore sstatus/sepc
//     ld t0, 32*8(sp)
//     ld t1, 33*8(sp)
//     csrw sstatus, t0
//     csrw sepc, t1
//     # restore general purpose registers except x0/sp/tp
//     ld x1, 1*8(sp)
//     ld x3, 3*8(sp)
//     .set n, 5
//     .rept 27
//         LOAD_GP %n
//         .set n, n+1
//     .endr
//     # back to user stack
//     ld sp, 2*8(sp)
//     sret"#
//     ).unwrap();