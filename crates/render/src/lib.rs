pub fn add(left: u64, right: u64) -> u64 {
    left + right // what is this for?
}

mod window;

pub use window::RenderWindow;
pub use wgpu;

pub const WINDOW_WIDTH: u32 = 800;
pub const WINDOW_HEIGHT: u32 = 600;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
