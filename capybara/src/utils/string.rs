pub trait StringUtils {
    fn as_bytes_array<const N: usize>(&self) -> [u8; N];
}

impl StringUtils for str {
    fn as_bytes_array<const N: usize>(&self) -> [u8; N] {
        let mut buffer = [0; N];
        let mut length = self.len();

        if self.len() > N {
            length = N;
        }

        buffer[..length].clone_from_slice(self[..length].as_bytes());
        buffer
    }
}
