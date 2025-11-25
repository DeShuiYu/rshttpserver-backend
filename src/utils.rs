pub(crate) fn format_bytes(b: u64) -> String {
    // 预定义单位常量 (Base 1024)
    const KIB: u64 = 1024;
    const MIB: u64 = 1024 * KIB;
    const GIB: u64 = 1024 * MIB;
    const TIB: u64 = 1024 * GIB;
    const PIB: u64 = 1024 * TIB; // 1 PiB

    match b {
        // 0. B (字节)
        0..KIB => {
            // 0 到 1023 字节，直接返回整数形式，不需要浮点数
            format!("{}B", b)
        }

        // 1. KiB (千字节)
        KIB..MIB => {
            format!("{:.1}K", b as f64 / KIB as f64)
        }

        // 2. MiB (兆字节)
        MIB..GIB => {
            format!("{:.1}M", b as f64 / MIB as f64)
        }

        // 3. GiB (千兆字节)
        GIB..TIB => {
            format!("{:.1}G", b as f64 / GIB as f64)
        }

        // 4. TiB (太字节)
        TIB..PIB => {
            format!("{:.1}T", b as f64 / TIB as f64)
        }

        _ => {
            // 对 TiB 以上的超大值统一处理，需除以 PiB 的基数
            format!("{:.1}P", b as f64 / PIB as f64)
        }
    }
}
