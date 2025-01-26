/// Page sizes possible to map
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(usize)]
pub enum PageSize {
    #[default]
    Size4K = 0x1000,
    #[cfg(target_arch = "x86")]
    Size4M = 0x400000,
    #[cfg(target_arch = "x86_64")]
    Size2M = 0x200000,
    #[cfg(target_arch = "x86_64")]
    Size1G = 0x40000000,
}

impl TryFrom<usize> for PageSize {
    type Error = ();

    fn try_from(size: usize) -> Result<Self, Self::Error> {
        match size {
            0x1000 => Ok(Self::Size4K),
            #[cfg(target_arch = "x86")]
            0x400000 => Ok(Self::Size4M),
            #[cfg(target_arch = "x86_64")]
            0x200000 => Ok(Self::Size2M),
            #[cfg(target_arch = "x86_64")]
            0x40000000 => Ok(Self::Size1G),
            _ => Err(()),
        }
    }
}

impl From<PageSize> for usize {
    fn from(value: PageSize) -> Self {
        value as _
    }
}

impl crate::memory::PageSizeTrait for PageSize {
    const MIN: Self = Self::Size4K;
}
