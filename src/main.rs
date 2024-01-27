
use std::io::Write;

mod error;
use error::{Error, Result};


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

/// Compilers
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
    X86_64,
}

/// Build mode
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

impl Default for Config {
    fn default() -> Self {
        Config {
            language: Language::C,
            standard: Standard::C89,
            compiler: Compiler::GCC,
            build_type: Type::Executable,
            target: Target::X86_64,
            mode: Mode::Debug,
        }
    }
}

struct Arguments {
    command: String,
    project_name: Option<String>,
    config: Config,
}

fn parse_arguments() -> Result<Arguments> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        return Err(Error::Error("Not enough arguments"));
    }

    let command = &args[1];
    match command.as_str() {
        "build" => {
            Ok(Arguments {
                command: command.clone(),
                project_name: None,
                config: Config::default(),
            })
        },
        "new" => {
            if args.len() < 3 {
                return Err(
                    Error::Error("Project name is required for `new` command")
                );
            }
            let project_name = args[2].clone();
            Ok(Arguments {
                command: command.clone(),
                project_name: Some(project_name),
                config: Config::default(),
            })
        },
        _ => Err(Error::Error("Unknown command")),
    }
}

fn create_new_project(project_name: &str, build_type: Type) -> Result<()> {
    // Create the project directory
    // @TODO: What if the directory already exists?
    std::fs::create_dir(project_name)?;
    
    // Create the project directory structure
    // @TODO: If the user asks for librarys, make `/lib`
    let src_path = format!("{}/src", project_name);
    let bin_path = format!("{}/bin", project_name);
    std::fs::create_dir(src_path.clone());
    std::fs::create_dir(bin_path);

    // Create default source files
    let main_file_path = format!("{}/main.c", src_path);
    let mut main_file = std::fs::File::create(main_file_path)?;
    writeln!(main_file, 
        "\nint main(int argc, char** argv) {{\n  return 0;\n}}"
    )?;

    // Create default configuration file
    let config_file_path = format!("{}/config.toml", project_name);
    let mut config_file = std::fs::File::create(config_file_path)?;
    writeln!(config_file, "[project]\nname = \"{}\"", project_name)?;

    Ok(())
}

fn main() {
    // @TODO: Print usage on argument fail
    let args = parse_arguments().unwrap();

    match args.command.as_str() {
        "build" => {
            println!("Build project");
        },
        "new" => {
            println!("New project: {}", args.project_name.unwrap());
        },
        _ => todo!(),
    }
}
