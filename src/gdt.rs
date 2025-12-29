use lazy_static::lazy_static;
use x86_64::{
    VirtAddr,
    instructions::tables::load_tss,
    registers::segmentation::{CS, Segment},
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
        tss::TaskStateSegment,
    },
};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

struct GlobalDescriptorContext {
    gdt: GlobalDescriptorTable,
    cs_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

lazy_static! {
    static ref TASK_STATE_SEGMENT: TaskStateSegment = {
        let mut task_state_segment = TaskStateSegment::new();
        task_state_segment.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            // essentially sets up a fake kernel stack for interrupt handlers
            // major disadvantage is that stack overflows are not guarded against
            const STACK_SIZE: usize = 4096 * 5;

            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(&raw const STACK);

            // since stacks grow downward, the address we start writing to is actually stack_start + STACK_SIZE
            stack_start + STACK_SIZE as u64
        };
        task_state_segment
    };
}

// GDT is needed to actually load the TSS
lazy_static! {
    static ref GLOBAL_DESCRIPTOR_CONTEXT: GlobalDescriptorContext = {
        let mut gdt = GlobalDescriptorTable::new();
        let cs_selector = gdt.append(Descriptor::kernel_code_segment());
        let tss_selector = gdt.append(Descriptor::tss_segment(&TASK_STATE_SEGMENT));
        GlobalDescriptorContext {
            gdt,
            cs_selector,
            tss_selector,
        }
    };
}

pub fn initialize_global_descriptor_table() {
    GLOBAL_DESCRIPTOR_CONTEXT.gdt.load();
    unsafe {
        CS::set_reg(GLOBAL_DESCRIPTOR_CONTEXT.cs_selector);
        load_tss(GLOBAL_DESCRIPTOR_CONTEXT.tss_selector);
    }
}
