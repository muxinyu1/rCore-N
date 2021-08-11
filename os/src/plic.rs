use rv_plic::Priority;
use rv_plic::PLIC;

use crate::trap::{push_trap_record, UserTrapRecord, USER_EXT_INT_MAP};
use crate::uart;

#[cfg(any(feature = "board_qemu", feature = "board_lrv"))]
pub const PLIC_BASE: usize = 0xc00_0000;
#[cfg(any(feature = "board_qemu", feature = "board_lrv"))]
pub const PLIC_PRIORITY_BIT: usize = 3;

pub type Plic = PLIC<PLIC_BASE, PLIC_PRIORITY_BIT>;

pub fn get_context(hartid: usize, mode: char) -> usize {
    const MODE_PER_HART: usize = 3;
    hartid * MODE_PER_HART
        + match mode {
            'M' => 0,
            'S' => 1,
            'U' => 2,
            _ => panic!("Wrong Mode"),
        }
}

pub fn handle_external_interrupt() {
    if let Some(irq) = Plic::claim(get_context(0, 'S')) {
        let mut can_user_handle = false;
        if let Some(pid) = USER_EXT_INT_MAP.lock().get(&irq) {
            debug!("[PLIC] irq {:?} mapped to pid {:?}",irq, pid);
            if let Ok(_) = push_trap_record(
                *pid,
                UserTrapRecord {
                    // User External Interrupt
                    cause: 8,
                    message: irq as usize,
                },
            ) {
                can_user_handle = true;
            }
        }
        if !can_user_handle {
            match irq {
                #[cfg(feature = "board_qemu")]
                12 => {
                    uart::handle_interrupt();
                    debug!("[PLIC] irq {:?} handled by kenel, UART2", irq);
                }
                #[cfg(feature = "board_lrv")]
                4 => {
                    uart::handle_interrupt();
                    debug!("[PLIC] kenel handling uart");
                }
                _ => {
                    warn!("[PLIC]: irq {:?} not supported!", irq);
                }
            }
        }
        Plic::complete(get_context(0, 'S'), irq)
    }
}

pub fn init() {
    Plic::set_threshold(1, Priority::any());
    Plic::set_threshold(2, Priority::any());
    #[cfg(feature = "board_qemu")]
    {
        Plic::enable(1, 12);
        Plic::set_priority(9, Priority::lowest());
        Plic::set_priority(10, Priority::lowest());
        Plic::set_priority(12, Priority::lowest());
    }
    #[cfg(feature = "board_lrv")]
    {
        Plic::enable(1, 3);
        Plic::set_priority(3, Priority::lowest());
        Plic::set_priority(4, Priority::lowest());
    }
}
