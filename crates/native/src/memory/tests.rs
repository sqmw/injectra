use super::*;
use crate::{ByteOrder, TargetArchitecture};
use injectra_core::BackendId;

// Only this test fixture has memory storage. It never touches a real process.
struct BufferSession {
    target: NativeTarget,
    caps: Vec<Capability>,
    bytes: Vec<u8>,
    calls: usize,
    limit: usize,
    lose_reply: bool,
}
impl BufferSession {
    fn new(bits: u8) -> Self {
        Self {
            target: NativeTarget {
                reference: TargetRef::new(&BackendId::fresh(), "process-instance-opaque"),
                pid: 42,
                architecture: TargetArchitecture::new("fixture", bits, ByteOrder::Little).unwrap(),
            },
            caps: vec![MEMORY_READ, MEMORY_WRITE],
            bytes: vec![0; 32],
            calls: 0,
            limit: 32,
            lose_reply: false,
        }
    }
}
impl NativeSession for BufferSession {
    fn target(&self) -> &NativeTarget {
        &self.target
    }
    fn capabilities(&self) -> &[Capability] {
        &self.caps
    }
}
impl MemoryRead for BufferSession {
    fn read(&mut self, address: TargetAddress, length: usize) -> MemoryResult<Vec<u8>> {
        self.calls += 1;
        let start = usize::try_from(address.0).unwrap();
        Ok(self
            .bytes
            .get(start..start + length.min(self.limit))
            .unwrap_or_default()
            .to_vec())
    }
}
impl MemoryWrite for BufferSession {
    fn write(&mut self, address: TargetAddress, bytes: &[u8]) -> MemoryResult<usize> {
        self.calls += 1;
        let start = usize::try_from(address.0).unwrap();
        let n = bytes.len().min(self.limit);
        self.bytes[start..start + n].copy_from_slice(&bytes[..n]);
        if self.lose_reply {
            return Err(MemoryError::write(
                Error::new(
                    ErrorKind::Connection,
                    Phase::Execute,
                    Certainty::Unknown,
                    "lost native reply",
                ),
                n,
            ));
        }
        Ok(n)
    }
}
#[test]
fn native_target_and_non_json_results_work_without_web_types() {
    let mut s = BufferSession::new(64);
    let target = s.target.reference.clone();
    assert_eq!(
        write_memory(&mut s, &target, TargetAddress(3), &[0, 255, 128]).unwrap(),
        3
    );
    assert_eq!(
        read_memory(&mut s, &target, TargetAddress(3), 3).unwrap(),
        [0, 255, 128]
    );
}
#[test]
fn identity_capability_and_target_width_reject_before_backend_calls() {
    let mut s = BufferSession::new(32);
    let target = s.target.reference.clone();
    let foreign = TargetRef::new(&BackendId::fresh(), target.key());
    assert_eq!(
        read_memory(&mut s, &foreign, TargetAddress(0), 1)
            .unwrap_err()
            .error
            .certainty,
        Certainty::NotExecuted
    );
    s.caps.clear();
    assert_eq!(
        read_memory(&mut s, &target, TargetAddress(0), 1)
            .unwrap_err()
            .error
            .kind,
        ErrorKind::Unsupported
    );
    s.caps = vec![MEMORY_READ];
    assert!(read_memory(&mut s, &target, TargetAddress(1u64 << 32), 1).is_err());
    assert!(read_memory(&mut s, &target, TargetAddress(u32::MAX.into()), 2).is_err());
    assert_eq!(s.calls, 0);
    let architecture = TargetArchitecture::new("64", 64, ByteOrder::Big).unwrap();
    assert!(
        architecture
            .validate_range(TargetAddress(u64::MAX), 1)
            .is_ok()
    );
    assert!(
        architecture
            .validate_range(TargetAddress(u64::MAX), 2)
            .is_err()
    );
}
#[test]
fn partial_transfers_keep_progress_and_unknown_writes_are_not_retried() {
    let mut s = BufferSession::new(64);
    let target = s.target.reference.clone();
    s.limit = 2;
    let partial = write_memory(&mut s, &target, TargetAddress(0), &[1, 2, 3, 4]).unwrap_err();
    assert_eq!(partial.transferred, 2);
    assert_eq!(partial.error.certainty, Certainty::Executed);
    assert_eq!(&s.bytes[..4], &[1, 2, 0, 0]);
    let read = read_memory(&mut s, &target, TargetAddress(0), 4).unwrap_err();
    assert_eq!(read.read_data, [1, 2]);
    assert_eq!(read.transferred, 2);
    s.lose_reply = true;
    let before = s.calls;
    let lost = write_memory(&mut s, &target, TargetAddress(0), &[9, 9, 9]).unwrap_err();
    assert_eq!(lost.error.certainty, Certainty::Unknown);
    assert_eq!(lost.transferred, 2);
    assert_eq!(s.calls, before + 1);
}
struct ReadOnly(BufferSession);
impl NativeSession for ReadOnly {
    fn target(&self) -> &NativeTarget {
        self.0.target()
    }
    fn capabilities(&self) -> &[Capability] {
        &[MEMORY_READ]
    }
}
impl MemoryRead for ReadOnly {
    fn read(&mut self, a: TargetAddress, n: usize) -> MemoryResult<Vec<u8>> {
        self.0.read(a, n)
    }
}
#[test]
fn read_only_backend_does_not_implement_write_port() {
    let mut s = ReadOnly(BufferSession::new(64));
    let target = s.target().reference.clone();
    assert_eq!(
        read_memory(&mut s as &mut dyn MemoryRead, &target, TargetAddress(0), 2).unwrap(),
        [0, 0]
    );
}
