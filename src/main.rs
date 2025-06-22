fn main() -> Result<(), String> {
    #[cfg(not(target_arch = "wasm32"))]
    libshaderand::native::glfw_main()?;

    Ok(())
}
