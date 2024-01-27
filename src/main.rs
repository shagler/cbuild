
/// Programming languages
enum Language {
    C,
}

/// Programming language standards
enum Standard {
    C89,
    C99,
    C11,
    C17
}

/// Build compilers
enum Compiler {
    /// GNU Compiler Collection (linux default)
    GCC,

    /// C/C++ LLVM Compiler
    CLANG,

    /// Microsoft Visual C++ (windows default)
    MSVC,
}

/// Build type
enum Type {
    /// (default)
    Executable,

    ///
    Library,

    /// 
    DynLibrary,
}

/// Build target
enum Target {
    X64_WINDOWS_MSVC,
}

enum Mode {
    /// (default)
    Debug,

    /// Debug symbols removed and optimization enabled
    Release,
}

struct Config {
    language: Language,
    standard: Standard,
    compiler: Compiler,
    build_type: Type,
    target: Target,
    mode: Mode,
}

struct Arguments {
    command: String,
    project_name: Option<String>,
    config: Config,
}

fn parse_arguments() -> Result<Arguments> {

}

fn main() {
    println!("Hello, world!");
}
