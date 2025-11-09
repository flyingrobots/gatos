# GATOS FFI Bindings

This crate provides a C-compatible Foreign Function Interface (FFI) for the GATOS core libraries. This allows GATOS to be integrated with other programming languages that can call into a C ABI (e.g., Python, Go, Ruby, etc.).

For more details on the overall architecture, see the main [GATOS Technical Specification](../../docs/TECH-SPEC.md).

## Building

To build the C-compatible shared library, run the following command from the root of the GATOS workspace:

```sh
car go build --release -p gatos-ffi-bindings
```

The resulting shared library will be located at `target/release/libgatos_ffi_bindings.so` (on Linux) or `target/release/libgatos_ffi_bindings.dylib` (on macOS).

## Usage Example

Here are basic examples of how to use the FFI from C and Python.

### C Example

You must link against the compiled shared library.

```c
#include <stdio.h>
#include <stdlib.h>

// Declare the functions from the Rust FFI library.
// In a real project, you would generate a header file for this.
const char* hello_ffi();
void gatos_ffi_free_string(char* s);

int main() {
    // Call the Rust function
    char* message = (char*)hello_ffi();
    if (message) {
        printf("Message from GATOS: %s\n", message);

        // IMPORTANT: Free the string that was allocated by Rust
        gatos_ffi_free_string(message);
    }

    return 0;
}
```

### Python Example (via `ctypes`)

```python
import ctypes
import platform

# Load the shared library
lib_name = ""
if platform.system() == "Linux":
    lib_name = "libgatos_ffi_bindings.so"
elif platform.system() == "Darwin":
    lib_name = "libgatos_ffi_bindings.dylib"
else:
    raise RuntimeError("Unsupported platform")

# Adjust the path to your target/release directory
lib_path = f"../../target/release/{lib_name}"
gatos_lib = ctypes.CDLL(lib_path)

# Define the function signatures
gatos_lib.hello_ffi.restype = ctypes.c_char_p
gatos_lib.gatos_ffi_free_string.argtypes = [ctypes.c_char_p]

# Call the Rust function
message_ptr = gatos_lib.hello_ffi()
print(f"Message from GATOS: {message_ptr.decode('utf-8')}")

# IMPORTANT: Free the string that was allocated by Rust
gatos_lib.gatos_ffi_free_string(message_ptr)
```

## Safety Considerations

Working with an FFI boundary requires careful attention to memory management and safety.

- **Ownership:** Any string or complex object returned by the GATOS FFI was allocated by Rust and its ownership is transferred to the caller. The caller is **required** to return control of the object to Rust for safe deallocation.
- **Memory Management:** All strings returned by this library **must** be freed by calling the provided `gatos_ffi_free_string` function. Failure to do so will result in memory leaks.
- **Null Pointers:** Do not pass `NULL` pointers to any function unless the function's documentation explicitly states that it is allowed.
- **Thread Safety:** The thread safety of the FFI functions is not guaranteed unless explicitly stated. Avoid calling FFI functions from multiple threads without proper synchronization.