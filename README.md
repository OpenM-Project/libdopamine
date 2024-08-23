<div align=center>
    <h1>libdopamine &#127769; &#9889;</h1>
</div>

-----

## :computer: Support
The library only works on Windows for now, but cross-platform support may be added in the future. ~~Don't look forward to it, though.~~

For all support needed to this library, you can open an [issue](https://github.com/wavEye-Project/libdopamine/issues/).

## :inbox_tray: Install
You can add the following code under `[dependencies]` in your `Cargo.toml` file:
```toml
libdopamine = { git = "https://github.com/wavEye-Project/libdopamine.git" }
```
:warning: **WARNING:** This will require you to have [`git`](https://git-scm.com/downloads) in your `PATH`.

## :zap: Example
In this small example, we will:
- Wait for a process to start & get a handle
- Wait for the module & get a handle
- Dump the module and patch it
- Inject patched module into memory

:warning: **WARNING:** This example ignores error handling using `.unwrap()`. Use the documentation for `libdopamine::errors` for information on how to handle errors.

```rust
use libdopamine;

fn main() {
    let (pid, process) = libdopamine::process::wait_for_process("my_app.exe").unwrap();
    let (_modname, module) = libdopamine::module::wait_for_module(process, "super_secret_stuff.dll").unwrap();
    let (length, mut data) = libdopamine::module::dump_module(process, module).unwrap();

    let mut new_data = ...; // Do your modifications here
    let _ = libdopamine::module::inject_module(process, module, &mut new_data, False);
}
```

## :page_with_curl: License
All code and assets are licensed under The Unlicense.
