use std::fs::File;
use std::io::{Read, Write};
use std::mem;

use orbclient::MouseEvent;

bitflags! {
    flags MousePacketFlags: u8 {
        const LEFT_BUTTON = 1,
        const RIGHT_BUTTON = 1 << 1,
        const MIDDLE_BUTTON = 1 << 2,
        const ALWAYS_ON = 1 << 3,
        const X_SIGN = 1 << 4,
        const Y_SIGN = 1 << 5,
        const X_OVERFLOW = 1 << 6,
        const Y_OVERFLOW = 1 << 7
    }
}

pub fn mouse(extra_packet: bool) {
    let mut file = File::open("irq:12").expect("ps2d: failed to open irq:12");
    let mut input = File::open("display:input").expect("ps2d: failed to open display:input");

    let mut packets = [0; 4];
    let mut packet_i = 0;
    loop {
        let mut irqs = [0; 8];
        if file.read(&mut irqs).expect("ps2d: failed to read irq:12") >= mem::size_of::<usize>() {
            let data: u8;
            unsafe {
                asm!("in al, dx" : "={al}"(data) : "{dx}"(0x60) : : "intel", "volatile");
            }

            file.write(&irqs).expect("ps2d: failed to write irq:12");

            packets[packet_i] = data;
            packet_i += 1;

            let flags = MousePacketFlags::from_bits_truncate(packets[0]);
            if ! flags.contains(ALWAYS_ON) {
                println!("MOUSE MISALIGN {:X}", packets[0]);

                packets = [0; 4];
                packet_i = 0;
            } else if packet_i >= packets.len() || (!extra_packet && packet_i >= 3) {
                if ! flags.contains(X_OVERFLOW) && ! flags.contains(Y_OVERFLOW) {
                    let mut dx = packets[1] as i32;
                    if flags.contains(X_SIGN) {
                        dx -= 0x100;
                    }

                    let mut dy = -(packets[2] as i32);
                    if flags.contains(Y_SIGN) {
                        dy += 0x100;
                    }

                    let _extra = if extra_packet {
                        packets[3]
                    } else {
                        0
                    };

                    input.write(&MouseEvent {
                        x: dx,
                        y: dy,
                        left_button: flags.contains(LEFT_BUTTON),
                        middle_button: flags.contains(MIDDLE_BUTTON),
                        right_button: flags.contains(RIGHT_BUTTON)
                    }.to_event()).expect("ps2d: failed to write mouse event");
                } else {
                    println!("ps2d: overflow {:X} {:X} {:X} {:X}", packets[0], packets[1], packets[2], packets[3]);
                }

                packets = [0; 4];
                packet_i = 0;
            }
        }
    }
}
