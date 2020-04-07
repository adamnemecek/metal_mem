

use metalgear::round_up;

#[test]
fn test_roundup() {
    assert!(round_up(1, 4096) == 4096);
    assert!(round_up(4095, 4096) == 4096);
    assert!(round_up(4096, 4096) == 4096);
    assert!(round_up(4097, 4096) == 2 * 4096);
    assert!(round_up(2 * 4096 + 1, 4096) == 3 * 4096);
}