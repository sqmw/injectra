use injectra_core::{Error, ErrorKind, Result, TargetRef};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteOrder {
    Little,
    Big,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetArchitecture {
    pub name: String,
    address_bits: u8,
    pub byte_order: ByteOrder,
}
impl TargetArchitecture {
    pub fn new(name: impl Into<String>, address_bits: u8, byte_order: ByteOrder) -> Result<Self> {
        if !(1..=64).contains(&address_bits) {
            return Err(Error::prepare(
                ErrorKind::InvalidInput,
                "target address width must be 1..64 bits",
            ));
        }
        Ok(Self {
            name: name.into(),
            address_bits,
            byte_order,
        })
    }
    pub fn address_bits(&self) -> u8 {
        self.address_bits
    }
    pub(crate) fn validate_range(&self, address: TargetAddress, length: usize) -> Result<()> {
        let limit = 1u128 << self.address_bits;
        if u128::from(address.0) >= limit || u128::from(address.0) + length as u128 > limit {
            return Err(Error::prepare(
                ErrorKind::InvalidInput,
                "memory range exceeds target address width",
            ));
        }
        Ok(())
    }
}
/// A target address is not a pointer in the controller's address space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetAddress(pub u64);

#[derive(Debug, Clone)]
pub struct NativeTarget {
    /// Provider must bind this opaque key to a process instance, not only its PID.
    pub reference: TargetRef,
    pub pid: u32,
    pub architecture: TargetArchitecture,
}
