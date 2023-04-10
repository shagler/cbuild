
# cbuild

`cbuild` is a modern build tool for C/C++ projects, inspired by Rust's `cargo`. It aims to simplify the build process and dependency management for C/C++ projects, with a focus on cross-platform compatibility and ease of use.

## Features

- Simple project setup with a single configuration file (`build.toml`)
- Automatic build configuration based on the `build.toml` settings
- Support for multiple languages (C, C++) and standards (C89, C99, C11, C17, C++98, C++11, C++14, C++17, C++20)
- Cross-platform support (Windows, Linux, macOS) and multiple compiler options (GCC, Clang, MSVC)
- Library dependency management
- Debug and release build modes

## Getting Started

### Installation

_TODO: Provide installation instructions for different platforms._

### Creating a New Project

To create a new project, run the following command:

```
cbuild new <project_name>
```

This will create a new directory with the given project name, along with the necessary files and directory structure:

```
<project_name>
├── build.toml
├── src
│   └── main.c (or main.cpp)
└── lib
└── bin
```

### Configuration

The `build.toml` file is used to configure your project's build settings. Here's an example of a `build.toml` file:

```toml
[project]
name = "example"

[build]
language = "c"
standard = "c89"
compiler = "gcc"
type = "bin"
target = "x86_64-windows-msvc"
mode = "debug"
compile_flags = ["-Wall", "-Wextra", "-Werror", "-pedantic"]
link_flags = ["lm"]
use_libc = true

[libraries]
core = "1.0.0"
```

### Building Your Project

To build your project, navigate to the project directory and run:

```
cbuild build
```

This will compile your project according to the settings in your `build.toml` file, and generate the output binary in a `/bin` directory.

## Contributing

_TODO: Provide information on how to contribute to the project, including any required style guides or code of conduct._

## License

_TODO: Specify the license under which the project is distributed._

## Acknowledgements

_TODO: List any third-party libraries, tools, or other resources used in the project._
