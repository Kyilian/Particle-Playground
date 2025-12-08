mod window;

pub use wgpu;
pub use window::RenderWindow;

pub const WINDOW_WIDTH: u32 = 800;
pub const WINDOW_HEIGHT: u32 = 600;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_dimensions() {
        assert_eq!(WINDOW_HEIGHT, 600);
        assert_eq!(WINDOW_WIDTH, 800);
    }

    #[test]
    fn test_window_dimensions_are_positive() {
        // Test that dimensions are valid positive values
        assert!(WINDOW_WIDTH > 0, "Window width must be positive");
        assert!(WINDOW_HEIGHT > 0, "Window height must be positive");
        
        // Test that dimensions are reasonable (not too small or too large)
        assert!(WINDOW_WIDTH >= 100, "Window width should be at least 100 pixels");
        assert!(WINDOW_HEIGHT >= 100, "Window height should be at least 100 pixels");
        assert!(WINDOW_WIDTH <= 10000, "Window width should be reasonable");
        assert!(WINDOW_HEIGHT <= 10000, "Window height should be reasonable");
    }
}