//! Terms of use text and hash.
//!
//! Displayed during first-run setup. User must accept before creating master key.

use olang::sha256::Sha256;

/// Terms of use — version 1.
pub const TERMS_TEXT: &str = "\
QUY TẮC SỬ DỤNG HomeOS

1. origin.olang là tài sản cá nhân của bạn.
   File này chứa MỌI THỨ: VM, logic, tri thức,
   khóa xác thực. MẤT FILE = MẤT HẾT.

2. HomeOS học từ bạn. Dữ liệu KHÔNG rời khỏi
   thiết bị. Không cloud. Không telemetry.
   Bạn sở hữu 100% dữ liệu của mình.

3. Append-only: HomeOS không xóa, không ghi đè.
   Mọi thay đổi được ghi lại vĩnh viễn.

4. HomeOS KHÔNG chịu trách nhiệm cho:
   - Quyết định dựa trên đề xuất của HomeOS
   - Mất file do lỗi phần cứng / người dùng
   - Hành vi của Worker trên thiết bị ngoại vi
   HomeOS là CÔNG CỤ. Người dùng quyết định.
   AAM approve = NGƯỜI DÙNG approve.

5. Backup: xuất key.ol để khôi phục trên máy khác.
   Không có key.ol → không khôi phục được.";

/// Compute hash of terms text for recording in AuthHeader.
pub fn terms_hash() -> u64 {
    let mut h = Sha256::new();
    h.update(TERMS_TEXT.as_bytes());
    let digest = h.finalize();
    u64::from_le_bytes(digest[0..8].try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terms_hash_deterministic() {
        let h1 = terms_hash();
        let h2 = terms_hash();
        assert_eq!(h1, h2);
        assert_ne!(h1, 0);
    }

    #[test]
    fn test_terms_text_not_empty() {
        assert!(TERMS_TEXT.len() > 100);
    }
}
