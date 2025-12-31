use lazy_static::lazy_static;
use x86_64::{VirtAddr, instructions::tables::load_tss, registers::segmentation::{CS, DS, ES, SS, Segment}, structures::{gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector}, tss::TaskStateSegment}};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

struct GlobalDescriptorContext {
    gdt: GlobalDescriptorTable,
    kernel_code: SegmentSelector,
    task_state: SegmentSelector,
    kernel_data: SegmentSelector,
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
        let kernel_code = gdt.append(Descriptor::kernel_code_segment());
        let task_state = gdt.append(Descriptor::tss_segment(&TASK_STATE_SEGMENT));
        let kernel_data = gdt.append(Descriptor::kernel_data_segment());
        
        GlobalDescriptorContext {
            gdt,
            kernel_code,
            task_state,
            kernel_data,
        }
    };
}

pub fn initialize_global_descriptor_table() {
    GLOBAL_DESCRIPTOR_CONTEXT.gdt.load();
    unsafe {
        CS::set_reg(GLOBAL_DESCRIPTOR_CONTEXT.kernel_code);
        SS::set_reg(GLOBAL_DESCRIPTOR_CONTEXT.kernel_data);
        DS::set_reg(GLOBAL_DESCRIPTOR_CONTEXT.kernel_data);
        ES::set_reg(GLOBAL_DESCRIPTOR_CONTEXT.kernel_data);
        load_tss(GLOBAL_DESCRIPTOR_CONTEXT.task_state);
    }
}
