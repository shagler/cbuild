
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
    /// Standard binary executable (default)
    Binary,

    /// `.lib` library file
    Library,

    /// `.dll` dynamic library file
    DynLibrary,
}

/// Build target
#[derive(PartialEq)]
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

struct Settings {
    language: Language,
    standard: Standard,
    compiler: Compiler,
    build_type: Type,
    target: Target,
    mode: Mode,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            language: Language::C,
            standard: Standard::C89,
            compiler: Compiler::GCC,
            build_type: Type::Binary,
            target: Target::X86_64,
            mode: Mode::Debug,
        }
    }
}

struct Config {
    project_name: Option<String>,
    settings: Settings,
}

impl Config {
    pub fn new(project_name: &str) -> Self {
        Config { 
            project_name: Some(project_name.to_string()), 
            settings: Settings::default() 
        }
    }

    pub fn load() -> Result<Self> {
        let working_directory = std::env::current_dir()?;
        let config_file = Self::find_config_file(&working_directory)?;
        
        let contents = std::fs::read_to_string(config_file)?;
        Ok(parse_config_toml(&contents)?)
    }

    fn find_config_file(path: &std::path::Path) -> Result<std::path::PathBuf> {
        let mut directories = vec![path.to_path_buf()];
        while let Some(dir) = directories.pop() {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        directories.push(path);
                    }
                    else if path.file_name().map_or(
                            false, |name| name == "config.toml") {
                        return Ok(path);
                    }
                }
            }
        }

        Err(Error::NoConfig())
    }
}

struct Arguments {
    command: String,
    config: Config,
}

fn parse_config_toml(config: &str) -> Result<Config> {
    let mut project_name = None;
    let mut settings = Settings::default();

    for line in config.lines() {
        let parts: Vec<&str> = line.split('=').map(|part| part.trim()).collect();
        if parts.len() == 2 {
            match parts[0] {
                "name" => {
                    project_name = Some(parts[1].trim_matches('"').to_string());
                },
                "language" => {
                    settings.language = match parts[1].trim_matches('"') {
                        "c" => Language::C,
                        _ => return Err(Error::Error("Unsupported language")),
                    }
                },
                "standard" => {
                    settings.standard = match parts[1].trim_matches('"') {
                        "c89" => Standard::C89,
                        "c99" => Standard::C99,
                        "c11" => Standard::C11,
                        "c17" => Standard::C17,
                        _ => return Err(Error::Error("Unsupported standard")),
                    }
                },
                "compiler" => {
                    settings.compiler = match parts[1].trim_matches('"') {
                        "gcc"   => Compiler::GCC,
                        "clang" => Compiler::CLANG,
                        "msvc"  => Compiler::MSVC,
                        _ => return Err(Error::Error("Unsupported compiler")),
                    }
                },
                "type" => {
                    settings.build_type = match parts[1].trim_matches('"') {
                        "bin"   => Type::Binary,
                        "lib"   => Type::Library,
                        "dylib" => Type::DynLibrary,
                        _ => return Err(Error::Error("Unsupported type")),
                    }
                },
                "target" => {
                    settings.target = match parts[1].trim_matches('"') {
                        "x86_64" => Target::X86_64,
                        _ => return Err(Error::Error("Unsupported target")),
                    }
                },
                "mode" => {
                    settings.mode = match parts[1].trim_matches('"') {
                        "debug"   => Mode::Debug,
                        "release" => Mode::Release,
                        _ => return Err(Error::Error("Unsupported mode")),
                    }
                },
                _ => (),
            }
        }
    }

    Ok(Config {
        project_name,
        settings,
    })
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
                config: Config::load()?,
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
                config: Config::new(&project_name),
            })
        },
        _ => Err(Error::Error("Unknown command")),
    }
}

fn create_new_project(project_name: &str) -> Result<()> {
    // Create the project directory
    // @TODO: What if the directory already exists?
    let prj_path = format!("tests/{}", project_name);
    std::fs::create_dir(prj_path)?;
    
    // Create the project directory structure
    // @TODO: If the user asks for librarys, make `/lib`
    let src_path = format!("tests/{}/src", project_name);
    let bin_path = format!("tests/{}/bin", project_name);
    std::fs::create_dir(src_path.clone());
    std::fs::create_dir(bin_path);

    // Create default source files
    let main_file_path = format!("{}/main.c", src_path);
    let mut main_file = std::fs::File::create(main_file_path)?;
    writeln!(main_file, 
        "\nint main(int argc, char** argv) {{\n  return 0;\n}}"
    )?;

    // Create default configuration file
    // @TODO: Use from Config::default()
    let config_file_path = format!("tests/{}/config.toml", project_name);
    let mut config_file = std::fs::File::create(config_file_path)?;
    writeln!(config_file, "[project]\nname = \"{}\"", project_name)?;

    Ok(())
}

fn build_project(config: Config) -> Result<()> {
    let project_name = config.project_name.as_ref().unwrap();
    let src_path = format!("tests/{}/src", project_name);
    let bin_path = format!("tests/{}/bin", project_name);
    let output_file = format!("{}/{}", bin_path, project_name);
    let src_file = format!("{}/main.c", src_path);

    let (compiler, mut args) = match config.settings.compiler {
        Compiler::GCC | Compiler::CLANG => {
            let compiler = match config.settings.compiler {
                Compiler::GCC => "gcc",
                Compiler::CLANG => "clang",
                _ => unreachable!(),
            };
            
            let mut args = vec!["-o", &output_file];

            args.push(match config.settings.standard {
                Standard::C89 => "-std=c89",
                Standard::C99 => "-std=c99",
                Standard::C11 => "-std=c11",
                Standard::C17 => "-std=c17",
            });

            match config.settings.mode {
                Mode::Debug   => args.push("-g"),
                Mode::Release => args.extend(["-O3", "-s"]),
            }

            if config.settings.target == Target::X86_64 {
                args.push("-m64");
            }
            
            args.push(src_file.as_str());

            (compiler, args)
        },
        Compiler::MSVC => {
            unimplemented!("MSVC compiler support not implemented yet");
        },
        _ => unreachable!(),
    };

    let args: Vec<String> = args.iter().map(|&arg| arg.to_string()).collect();

    let output = std::process::Command::new(compiler)
        .args(&args)
        .output()
        .expect("Failed to execute build command");

    if !output.status.success() {
        std::io::stderr().write_all(&output.stderr)?;
        return Err(Error::BuildFailed());
    }
    
    println!("Built `test_project`");
    Ok(())
}

fn load_test_project() -> Result<()> {
    let config_file = std::path::Path::new("tests/config.toml");
    let config_file = std::fs::read_to_string(config_file)?;
    let config = parse_config_toml(&config_file)?;
    println!("Loaded `tests/config.toml`");

    let project = create_new_project("test_project");

    let build = build_project(config)?;

    Ok(())
}

fn main() -> Result<()> {
    // @TODO: Print usage on argument fail
    let args = parse_arguments().unwrap();

    match args.command.as_str() {
        "build" => {
            println!("Build project");
            let prj = load_test_project()?;
        },
        "new" => {
            println!("New project: {}", args.config.project_name.unwrap());
        },
        _ => todo!(),
    }

    Ok(())
}
