//! UI の共通部品（座標定数・見出し帯・タイル・文字列整形）。

pub mod art;
pub mod image;
pub mod layout;
pub mod widgets;

/// 固定長フォーマットバッファ（no-alloc で `write!` を受ける）。あふれた分は切り捨てる。
pub struct FmtBuf<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> FmtBuf<N> {
    /// 空のバッファ。
    pub fn new() -> Self {
        Self {
            buf: [0; N],
            len: 0,
        }
    }

    /// 中身を空にする。
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// 書き込んだ文字列。
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }
}

impl<const N: usize> core::fmt::Write for FmtBuf<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let n = bytes.len().min(N - self.len);
        self.buf[self.len..self.len + n].copy_from_slice(&bytes[..n]);
        self.len += n;
        Ok(())
    }
}
