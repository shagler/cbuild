# cbuild
`cbuild` is a modern build tool for C/C++ projects, inspired by Rust's `cargo`. It aims to simplify the build process and dependency management for C/C++ projects, with a focus on cross-platform compatibility and ease of use.

## Features
- Simple project setup with `cbuild new {project_name}`
- Automatic build configuration based on the `config.toml` settings file
- Support for multiple languages (C, C++) and standards (C89, C99, C11, C17, C++98, C++11, C++14, C++17, C++20) 
- Cross-platform support (Windows/Linux) and multiple compiler options (GCC, Clang, MSVC)
- Basic library dependency management (header-only libraries)
- Debug and release build modes
- Verbose logging option for detailed build information

## Installation

### Prerequisites

- Rust and Cargo (latest stable version)
- C/C++ compiler (GCC, Clang, or MSVC depending on your platform and preference)

### Building from source

1. Clone the repository:
```
git clone https://github.com/shagler/cbuild.git
cd cbuild/
```

2. Build the project:
```
cargo build --release
```

3. The binary will be available in `target/release/cbuild`. You can move this to a directory in your PATH for easy access.

## Usage

### Creating a new project

To create a new project, run:
```
cbuild new {project_name}
```

This will create a new directory with the given project name, along with the necessary files and directory structure:
```
{project_name}
├── config.toml
├── src
│   └── main.c (or main.cpp)
└── lib
└── bin
```

### Configuration

The `config.toml` file is used to configure your project's build settings. Here's an example:
```toml
[project]
name = "my_project"

[settings]
language = "c"
standard = "c11"
compiler = "gcc"
type = "bin"
target = "x86_64"
mode = "debug"

libraries = ["mylib1", "mylib2"]
```

### Building your project

To build your project, navigate to the project directory and run:
```
cbuild build
```

This will compile your project according to the settings in your `config.toml` file, and generate the output binary in the `/bin` directory.

### Running your project

To build and run your project in one step, use:
```
cbuild run
```

### Cleaning build artifacts

To remove build artifacts, use:
```
cbuild clean
```

### Verbose output

For more detailed output during any command, add the `-v` or `--verbose` flag:
```
cbuild build --verbose
```

### Help

To see all available commands and options:
```
cbuild help
```

### Library management:

`cbuild` supports basic management of header-only libraries. Place your libraries in the `~/.cbuild/libs` directory. When specified in your `config.toml`, `cbuild` will copy these libraries to your project's `lib/` directory and include them in the build process.
