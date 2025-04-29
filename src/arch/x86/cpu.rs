pub struct Cpu;

impl crate::arch::CpuTrait for Cpu {
    fn cpu_id() -> usize {
        // TODO: Proper CPU id
        0
    }
}
