fn main() -> Result<(), String> {
    #[cfg(not(target_arch = "wasm32"))]
    shaderand::native::glfw_main()?;

    Ok(())
}
