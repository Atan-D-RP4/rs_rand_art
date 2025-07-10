fn main() -> Result<(), String> {
    #[cfg(not(target_arch = "wasm32"))]
    shaderand_wasm::native::glfw_main()?;

    Ok(())
}
