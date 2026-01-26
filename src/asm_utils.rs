// Assembly utilities for performance-critical operations in patch generation

/// Fast string sanitization using inline assembly
/// This is used to sanitize patch filenames and subject lines
pub fn fast_sanitize_string(input: &str) -> String {
    // For now, use Rust implementation, but this can be optimized with assembly
    // when dealing with very large strings
    input
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '_' || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

/// Calculate a simple checksum for patch validation
/// Uses inline assembly for the core calculation
pub fn calculate_patch_checksum(data: &[u8]) -> u32 {
    unsafe {
        let mut sum: u32 = 0;
        let len = data.len();
        let ptr = data.as_ptr();

        // Inline assembly for fast checksum calculation
        #[cfg(target_arch = "x86_64")]
        {
            core::arch::asm!(
                "xor eax, eax",
                "xor ecx, ecx",
                "2:",
                "cmp rcx, rdx",
                "je 3f",
                "movzx r8d, byte ptr [rsi + rcx]",
                "add eax, r8d",
                "inc rcx",
                "jmp 2b",
                "3:",
                in("rsi") ptr,
                in("rdx") len,
                out("rax") sum,
                out("rcx") _,
                out("r8") _,
            );
        }

        #[cfg(target_arch = "aarch64")]
        {
            core::arch::asm!(
                "mov x2, #0",
                "mov w3, #0",
                "2:",
                "cmp x2, x1",
                "b.eq 3f",
                "ldrb w4, [x0, x2]",
                "add w3, w3, w4",
                "add x2, x2, #1",
                "b 2b",
                "3:",
                in("x0") ptr,
                in("x1") len,
                out("x2") _,
                out("w3") sum,
                out("w4") _,
            );
        }

        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            // Fallback to Rust implementation
            sum = data.iter().map(|&b| b as u32).sum();
        }

        sum
    }
}

/// Fast line counting using assembly optimizations
pub fn fast_count_lines(data: &[u8]) -> usize {
    unsafe {
        let mut count: usize = 0;
        let len = data.len();
        let ptr = data.as_ptr();

        #[cfg(target_arch = "x86_64")]
        {
            core::arch::asm!(
                "xor rax, rax",
                "xor rcx, rcx",
                "2:",
                "cmp rcx, rdx",
                "je 3f",
                "cmp byte ptr [rsi + rcx], 0x0a",
                "jne 4f",
                "inc rax",
                "4:",
                "inc rcx",
                "jmp 2b",
                "3:",
                in("rsi") ptr,
                in("rdx") len,
                out("rax") count,
                out("rcx") _,
            );
        }

        #[cfg(target_arch = "aarch64")]
        {
            core::arch::asm!(
                "mov x2, #0",
                "mov x3, #0",
                "2:",
                "cmp x2, x1",
                "b.eq 3f",
                "ldrb w4, [x0, x2]",
                "cmp w4, #0x0a",
                "b.ne 4f",
                "add x3, x3, #1",
                "4:",
                "add x2, x2, #1",
                "b 2b",
                "3:",
                in("x0") ptr,
                in("x1") len,
                out("x2") _,
                out("x3") count,
                out("w4") _,
            );
        }

        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            // Fallback to Rust implementation
            count = data.iter().filter(|&&b| b == b'\n').count();
        }

        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum() {
        let data = b"test data";
        let checksum = calculate_patch_checksum(data);
        assert!(checksum > 0);
    }

    #[test]
    fn test_line_count() {
        let data = b"line1\nline2\nline3";
        let count = fast_count_lines(data);
        assert_eq!(count, 2);
    }
}
