# cbuild
`cbuild` is a modern build tool for C/C++ projects, inspired by Rust's `cargo`. It aims to simplify the build process and dependency management for C/C++ projects, with a focus on cross-platform compatbility and ease of use.

## Features (WIP)
- Simple project setup with `cbuild new {project_name}`
- Automatic build configuration based on the `config.toml` settings file
- Support for multiple languages (C, C++) and standards (C89, C99, C11, C17, C++98, C++11, C++14, C++17, C++20) 
- Cross-platform support (Windows/Linux) and multiple compiler options (GCC, Clang, MSVC)
- Library dependency management (single-headers within `lib`)
- Debug and release build modes

## Getting Started
### Installation
```
git clone https://github.com/shawnha/cbuild.git
cd cbuild/
cargo build
```
_TODO: `cargo install`_

### Creating a new project
To create a new project, run the following command:
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

The `config.toml` file is used to configure your project's build settings. Here's an example of a `config.toml` file:
```
[project]
name = "test_project"

[settings]
language = "c"
standard = "c89"
compiler = "gcc"
type = "bin"
target = "x86_64"
mode = "debug"
```

### Building your project

To build your project, navigate to the project directory and run:
```
cbuild build
```

This will compile your project according to the settings in you `config.toml` file, and generate the output binary in the `/bin` directory.
