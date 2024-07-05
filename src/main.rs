
use std::io::Write;

mod error;
use error::{Error, Result};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const GLOBAL_LIB_PATH: &str = "~/.cbuild/libs/";

/// Programming languages
#[derive(Clone, Debug)]
enum Language {
    C,
    CPP,
}

/// Programming language standards
#[derive(Clone, Debug)]
enum Standard {
    C89,
    C99,
    C11,
    C17,
    CPP98,
    CPP11,
    CPP14,
    CPP17,
    CPP20,
}

/// Compilers
#[derive(Clone, Debug)]
enum Compiler {
    /// GNU Compiler Collection (linux default)
    GCC,

    /// C/C++ LLVM Compiler
    CLANG,

    /// Microsoft Visual C++ (windows default)
    MSVC,
}

/// Build type
#[derive(Clone, Debug)]
enum Type {
    /// Standard binary executable (default)
    Binary,

    /// `.lib` library file
    Library,

    /// `.dll` dynamic library file
    DynLibrary,
}

/// Build target
#[derive(Clone, Debug, PartialEq)]
enum Target {
    X86_64,
}

/// Build mode
#[derive(Clone, Debug)]
enum Mode {
    /// (default)
    Debug,

    /// Debug symbols removed and optimization enabled
    Release,
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
struct Config {
    project_name: Option<String>,
    settings: Settings,
    libraries: Vec<String>,
    verbose: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            project_name: None,
            settings: Settings::default(),
            libraries: Vec::new(),
            verbose: false,
        }
    }
}

impl Config {
    pub fn new(project_name: &str) -> Self {
        Config { 
            project_name: Some(project_name.to_string()), 
            settings: Settings::default(),
            libraries: Vec::new(),
            verbose: false,
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

#[derive(Clone, Debug)]
struct Arguments {
    command: String,
    config: Config,
}

fn parse_config_toml(config: &str) -> Result<Config> {
    let mut project_name = None;
    let mut settings = Settings::default();
    let mut libraries = Vec::new();

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
                        "CPP" => Language::CPP,
                        _ => return Err(Error::Config("Unsupported language".to_string())),
                    }
                },
                "standard" => {
                    settings.standard = match parts[1].trim_matches('"') {
                        "c89" => Standard::C89,
                        "c99" => Standard::C99,
                        "c11" => Standard::C11,
                        "c17" => Standard::C17,
                        "CPP98" => Standard::CPP98,
                        "CPP11" => Standard::CPP11,
                        "CPP14" => Standard::CPP14,
                        "CPP17" => Standard::CPP17,
                        "CPP20" => Standard::CPP20,
                        _ => return Err(Error::Config("Unsupported standard".to_string())),
                    }
                },
                "compiler" => {
                    settings.compiler = match parts[1].trim_matches('"') {
                        "gcc"   => Compiler::GCC,
                        "clang" => Compiler::CLANG,
                        "msvc"  => Compiler::MSVC,
                        _ => return Err(Error::Config("Unsupported compiler".to_string())),
                    }
                },
                "type" => {
                    settings.build_type = match parts[1].trim_matches('"') {
                        "bin"   => Type::Binary,
                        "lib"   => Type::Library,
                        "dylib" => Type::DynLibrary,
                        _ => return Err(Error::Config("Unsupported type".to_string())),
                    }
                },
                "target" => {
                    settings.target = match parts[1].trim_matches('"') {
                        "x86_64" => Target::X86_64,
                        _ => return Err(Error::Config("Unsupported target".to_string())),
                    }
                },
                "mode" => {
                    settings.mode = match parts[1].trim_matches('"') {
                        "debug"   => Mode::Debug,
                        "release" => Mode::Release,
                        _ => return Err(Error::Config("Unsupported mode".to_string())),
                    }
                },
                "libraries" => {
                    libraries = parts[1].trim_matches('"')
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect();
                },
                _ => (),
            }
        }
    }

    Ok(Config {
        project_name,
        settings,
        libraries,
        verbose: false,
    })
}

fn parse_arguments() -> Result<Arguments> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        return Err(Error::Arguments("Not enough arguments".to_string()));
    }

    let command = &args[1];
    let mut config = match command.as_str() {
        "build" | "run" | "clean" => Config::load()?,
        "new" => {
            if args.len() < 3 {
                return Err(Error::Arguments("Project name is required for `new` command".to_string()));
            }
            let project_name = args[2].clone();
            Config::new(&project_name)
        },
        "help" | "version" => Config::default(),
        _ => return Err(Error::Arguments("Unknown command".to_string())),
    };

    config.verbose = args.contains(&"--verbose".to_string()) || args.contains(&"-v".to_string());

    Ok(Arguments {
        command: command.clone(),
        config,
    })
}

fn create_new_project(project_name: &str) -> Result<()> {
    let prj_path = std::path::Path::new(project_name);
    if prj_path.exists() {
        return Err(Error::ProjectCreation("Project directory already exists".to_string()));
    }
    std::fs::create_dir(prj_path)?;
    
    for dir in &["src", "bin", "lib"] {
        std::fs::create_dir(prj_path.join(dir))?;
    }
    
    // Create default source files
    let main_file_path = prj_path.join("src").join("main.c");
    let mut main_file = std::fs::File::create(main_file_path)?;
    writeln!(main_file, "#include <stdio.h>\n\nint main(int argc, char** argv) {{\n  printf(\"Hello, World!\\n\");\n  return 0;\n}}")?;

    let config_file_path = prj_path.join("config.toml");
    let mut config_file = std::fs::File::create(config_file_path)?;
    writeln!(config_file, "[project]\nname = \"{}\"\n\n[settings]\nlanguage = \"c\"\nstandard = \"c89\"\ncompiler = \"gcc\"\ntype = \"bin\"\ntarget = \"x86_64\"\nmode = \"debug\"", project_name)?;

    let gitignore_path = prj_path.join(".gitignore");
    let mut gitignore_file = std::fs::File::create(gitignore_path)?;
    writeln!(gitignore_file, "/bin\n*.o\n*.a\n*.so\n*.dll")?;

    println!("Created project: {}", project_name);

    Ok(())
}

fn manage_dependencies(config: &Config) -> Result<()> {
    log(config, "Managing dependencies");
    let project_lib_path = std::path::PathBuf::from(format!("{}/lib", config.project_name.as_ref().unwrap()));
    std::fs::create_dir_all(&project_lib_path)?;

    let global_lib_path = shellexpand::tilde(GLOBAL_LIB_PATH);

    for lib in &config.libraries {
        let global_lib_file = std::path::PathBuf::from(global_lib_path.to_string()).join(lib);
        let project_lib_file = project_lib_path.join(lib);

        if global_lib_file.exists() {
            if !project_lib_file.exists() {
                std::fs::copy(&global_lib_file, &project_lib_file)?;
                println!("Copied dependency: {} to project", lib);
            } else {
                println!("Dependency {} already exists in project", lib);
            }
        } else {
            return Err(Error::Library(format!("Library {} not found in global library path", lib)));
        }
    }

    Ok(())
}

fn build_project(config: Config) -> Result<()> {
    log(&config, "Starting build process");
    manage_dependencies(&config)?;

    let current_dir = std::env::current_dir()?;
    let src_path = current_dir.join("src");
    let lib_path = current_dir.join("lib");
    let bin_path = current_dir.join("bin");
    std::fs::create_dir_all(&bin_path)?;

    let project_name = config.project_name.as_ref().ok_or_else(|| Error::Config("Project name not found".to_string()))?;
    let output_file = bin_path.join(project_name);

    let mut source_files = Vec::new();
    for entry in std::fs::read_dir(&src_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "c" || ext == "cpp") {
            source_files.push(path);
        }
    }

    if source_files.is_empty() {
        return Err(Error::Config("No source files found in src directory".to_string()));
    }

    let mut args = Vec::new();
    let compiler = match config.settings.compiler {
        Compiler::GCC | Compiler::CLANG => {
            let compiler = match config.settings.compiler {
                Compiler::GCC => "gcc",
                Compiler::CLANG => "clang",
                _ => unreachable!(),
            };
            
            args.push(format!("-o{}", output_file.to_str().unwrap()));
            args.push(format!("-I{}", lib_path.to_str().unwrap()));

            args.push(match config.settings.standard {
                Standard::C89 => "-std=c89".to_string(),
                Standard::C99 => "-std=c99".to_string(),
                Standard::C11 => "-std=c11".to_string(),
                Standard::C17 => "-std=c17".to_string(),
                Standard::CPP98 => "-std=c++98".to_string(),
                Standard::CPP11 => "-std=c++11".to_string(),
                Standard::CPP14 => "-std=c++14".to_string(),
                Standard::CPP17 => "-std=c++17".to_string(),
                Standard::CPP20 => "-std=c++20".to_string(),
            });

            match config.settings.mode {
                Mode::Debug => args.push("-g".to_string()),
                Mode::Release => {
                    args.push("-O3".to_string());
                    args.push("-s".to_string());
                },
            }

            if config.settings.target == Target::X86_64 {
                args.push("-m64".to_string());
            }

            // Add source files to args
            args.extend(source_files.iter().map(|path| path.to_str().unwrap().to_string()));

            compiler
        },
        Compiler::MSVC => {
            let compiler = "cl.exe";
            args.push(format!("/Fe:{}", output_file.to_str().unwrap()));
            args.push(format!("/I{}", lib_path.to_str().unwrap()));

            args.push(match config.settings.standard {
                Standard::C89 => "/Za".to_string(),
                Standard::C99 | Standard::C11 | Standard::C17 => "/std:c11".to_string(),
                Standard::CPP98 | Standard::CPP11 | Standard::CPP14 => "/std:c++14".to_string(),
                Standard::CPP17 => "/std:c++17".to_string(),
                Standard::CPP20 => "/std:c++latest".to_string(),
            });

            match config.settings.mode {
                Mode::Debug => args.push("/Zi".to_string()),
                Mode::Release => {
                    args.push("/O2".to_string());
                    args.push("/DNDEBUG".to_string());
                },
            }

            if config.settings.target == Target::X86_64 {
                args.push("/MACHINE:X64".to_string());
            }

            // Add source files to args
            args.extend(source_files.iter().map(|path| path.to_str().unwrap().to_string()));

            compiler
        },
    };

    log(&config, &format!("Running command: {} {}", compiler, args.join(" ")));

    let output = std::process::Command::new(compiler)
        .args(&args)
        .output()
        .expect("Failed to execute build command");

    if !output.status.success() {
        std::io::stderr().write_all(&output.stderr)?;
        return Err(Error::BuildFailed());
    }
    
    println!("Built `{}`", project_name);
    Ok(())
}

fn run_project(config: &Config) -> Result<()> {
    log(config, "Running project");
    let project_name = config.project_name.as_ref().ok_or_else(|| Error::Config("Project name not found".to_string()))?;
    let current_dir = std::env::current_dir()?;
    let bin_path = current_dir.join("bin").join(project_name);
    
    if !bin_path.exists() {
        return Err(Error::Config(format!("Binary not found at: {}", bin_path.display())));
    }

    log(config, &format!("Attempting to run: {}", bin_path.display()));

    let output = std::process::Command::new(&bin_path)
        .output()
        .map_err(|e| Error::IO(e))?;

    std::io::stdout().write_all(&output.stdout)?;
    std::io::stderr().write_all(&output.stderr)?;

    if !output.status.success() {
        return Err(Error::RunFailed(output.status.code()));
    }

    Ok(())
}

fn clean_project() -> Result<()> {
    let bin_path = "bin";
    if std::path::Path::new(bin_path).exists() {
        std::fs::remove_dir_all(bin_path)?;
        println!("Cleaned build artifacts");
    }
    Ok(())
}

fn print_help() {
    println!("Usage: cbuild <COMMAND>");
    println!("\nCommands:");
    println!("  new <NAME>    Create a new project");
    println!("  build         Build the project");
    println!("  run           Build and run the project");
    println!("  clean         Remove build artifacts");
    println!("  version       Print version info");
    println!("  help          Print this help message");
    println!("\nOptions:");
    println!("  -v, --verbose Enable verbose output");
}

fn log(config: &Config, message: &str) {
    if config.verbose {
        println!("[cbuild] {}", message);
    }
}

fn main() -> Result<()> {
    let args = parse_arguments()?;

    let result = match args.command.as_str() {
        "build" => build_project(args.config),
        "new" => create_new_project(&args.config.project_name.unwrap()),
        "run" => {
            build_project(args.config.clone()).and_then(|_| run_project(&args.config))
        },
        "clean" => clean_project(),
        "version" => {
            println!("cbuild version {}", VERSION);
            Ok(())
        },
        "help" => {
            print_help();
            Ok(())
        },
        _ => Err(Error::Arguments("Unknown command".to_string())),
    };

    if let Err(e) = result {
        eprintln!("Error: {:?}", e);
        std::process::exit(1);
    }

    Ok(())
}