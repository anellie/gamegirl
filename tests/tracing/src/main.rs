use core::{common::SystemConfig, components::arm::Cpu, gga::GameGirlAdv, numutil::NumExt, System};
use std::{sync::mpsc, thread};

fn main() {
    env_logger::init();
    let mut non_cached = System::default();
    let mut cached = System::default();

    let rom = include_bytes!("../../../bench.gb").to_vec();
    cached.load_cart(rom.clone(), None, &SystemConfig::default());
    non_cached.load_cart(
        rom,
        None,
        &SystemConfig {
            cached_interpreter: false,
            ..SystemConfig::default()
        },
    );

    let (c_tx, c_rx) = mpsc::channel();
    let (n_tx, n_rx) = mpsc::channel();

    cached.gga_mut().cpu.instruction_tracer = Some(Box::new(move |gg, inst| {
        c_tx.send((gg.cpu.registers, gg.cpu.cpsr, inst)).unwrap();
    }));
    non_cached.gga_mut().cpu.instruction_tracer = Some(Box::new(move |gg, inst| {
        n_tx.send((gg.cpu.registers, gg.cpu.cpsr, inst)).unwrap();
    }));

    thread::spawn(move || loop {
        let cached_state = c_rx.recv().unwrap();
        let uncached_state = n_rx.recv().unwrap();

        for (i, (reg_c, reg_n)) in cached_state
            .0
            .iter()
            .zip(uncached_state.0.iter())
            .enumerate()
        {
            if reg_c != reg_n {
                eprintln!("R{i} mismatch! Expected {reg_n:X}, got {reg_c:X}");
                return;
            }
        }

        let mnem = if !cached_state.1.is_bit(5) {
            Cpu::<GameGirlAdv>::get_mnemonic_arm(uncached_state.2)
        } else {
            Cpu::<GameGirlAdv>::get_mnemonic_thumb(uncached_state.2.u16())
        };
        eprintln!("0x{:08X} {}", cached_state.0[15], mnem);
    });

    thread::spawn(move || loop {
        cached.advance();
    });

    loop {
        non_cached.advance();
    }
}
