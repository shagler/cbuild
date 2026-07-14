use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;

use serde::{Deserialize, Deserializer};

mod error;
use error::{Error, Result};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const GLOBAL_LIB_PATH: &str = "~/.cbuild/libs/";
const TEMP_BUILD_DIR: &str = "./.cbuild";

/// Programming languages
#[derive(Clone, Debug, Deserialize)]
#[serde(try_from = "String")]
enum Language {
    C,
    CPP,
}

/// Programming language standards
#[derive(Clone, Debug, Deserialize)]
#[serde(try_from = "String")]
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
    CPP23,
}

/// Compilers
#[derive(Clone, Debug, Deserialize)]
#[serde(try_from = "String")]
enum Compiler {
    /// GNU Compiler Collection (linux default)
    GCC,

    /// C/C++ LLVM Compiler
    CLANG,

    /// Microsoft Visual C++ (windows default)
    MSVC,
}

/// Build type
#[derive(Clone, Debug, Deserialize)]
#[serde(try_from = "String")]
enum Type {
    /// Standard binary executable (default)
    Binary,

    /// `.lib` static library file
    Library,

    /// `.dll` dynamic library file
    DynLibrary,
}

/// Build target
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(try_from = "String")]
enum Target {
    X86_64,
}

/// Build mode
#[derive(Clone, Debug, Deserialize)]
#[serde(try_from = "String")]
enum Mode {
    /// (default)
    Debug,

    /// Debug symbols removed and optimization enabled
    Release,
}

fn norm(s: &str) -> String {
    s.trim()
        .to_ascii_lowercase()
        .replace(['-', '_', ' '], "")
        .replace("cxx", "cpp")
}

impl TryFrom<String> for Language {
    type Error = String;
    fn try_from(s: String) -> std::result::Result<Self, Self::Error> {
        match norm(&s).as_str() {
            "c" => Ok(Language::C),
            "cpp" | "c++" | "cc" => Ok(Language::CPP),
            other => Err(format!("unsupported language: `{other}`")),
        }
    }
}

impl TryFrom<String> for Standard {
    type Error = String;
    fn try_from(s: String) -> std::result::Result<Self, Self::Error> {
        match norm(&s).as_str() {
            "c89" | "c90" => Ok(Standard::C89),
            "c99" => Ok(Standard::C99),
            "c11" => Ok(Standard::C11),
            "c17" | "c18" => Ok(Standard::C17),
            "c++98" | "cpp98" | "c++03" | "cpp03" => Ok(Standard::CPP98),
            "c++11" | "cpp11" | "c++0x" => Ok(Standard::CPP11),
            "c++14" | "cpp14" | "c++1y" => Ok(Standard::CPP14),
            "c++17" | "cpp17" | "c++1z" => Ok(Standard::CPP17),
            "c++20" | "cpp20" | "c++2a" => Ok(Standard::CPP20),
            "c++23" | "cpp23" | "c++2b" => Ok(Standard::CPP23),
            other => Err(format!("unsupported standard: `{other}`")),
        }
    }
}

impl TryFrom<String> for Compiler {
    type Error = String;
    fn try_from(s: String) -> std::result::Result<Self, Self::Error> {
        match norm(&s).as_str() {
            "gcc" | "g++" => Ok(Compiler::GCC),
            "clang" | "clang++" => Ok(Compiler::CLANG),
            "msvc" | "cl" => Ok(Compiler::MSVC),
            other => Err(format!("unsupported compiler: `{other}`")),
        }
    }
}

impl TryFrom<String> for Type {
    type Error = String;
    fn try_from(s: String) -> std::result::Result<Self, Self::Error> {
        match norm(&s).as_str() {
            "bin" | "exe" | "executable" => Ok(Type::Binary),
            "lib" | "static" | "staticlib" => Ok(Type::Library),
            "dylib" | "dll" | "shared" | "so" => Ok(Type::DynLibrary),
            other => Err(format!("unsupported type: `{other}`")),
        }
    }
}

impl TryFrom<String> for Target {
    type Error = String;
    fn try_from(s: String) -> std::result::Result<Self, Self::Error> {
        match norm(&s).as_str() {
            // Normalization strips `_` so `x86_64` = `x8664`
            "x8664" | "x64" | "amd64" => Ok(Target::X86_64),
            other => Err(format!("unsupported target: `{other}`")),
        }
    }
}

impl TryFrom<String> for Mode {
    type Error = String;
    fn try_from(s: String) -> std::result::Result<Self, Self::Error> {
        match norm(&s).as_str() {
            "debug" | "dev" => Ok(Mode::Debug),
            "release" | "rel" => Ok(Mode::Release),
            other => Err(format!("unsupported mode: `{other}`")),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
#[allow(dead_code)]
struct Settings {
    language: Language,
    standard: Standard,
    compiler: Compiler,
    #[serde(rename = "type")]
    build_type: Type,
    target: Target,
    mode: Mode,

    include_dirs: Vec<String>,
    defines: Vec<String>,
    #[serde(deserialize_with = "string_or_vec")]
    cflags: Vec<String>,
    #[serde(deserialize_with = "string_or_vec")]
    lflags: Vec<String>,
    system_libs: Vec<String>,
    #[serde(deserialize_with = "string_or_vec")]
    libraries: Vec<String>,
    allow_post_build_run: bool,
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
            include_dirs: Vec::new(),
            defines: Vec::new(),
            cflags: Vec::new(),
            lflags: Vec::new(),
            system_libs: Vec::new(),
            libraries: Vec::new(),
            allow_post_build_run: false,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
struct Project {
    name: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
#[allow(dead_code)]
struct Dependency {
    path: Option<String>,
    copy: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
#[allow(dead_code)]
enum PostBuild {
    Copy { from: String, to: String },
    Mkdir { path: String },
    Remove { path: String },
    Run { command: Vec<String> },
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
#[allow(dead_code)]
struct Workspace {
    members: Vec<String>,
}

fn string_or_vec<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrVec {
        Str(String),
        Vec(Vec<String>),
    }
    Ok(match StringOrVec::deserialize(d)? {
        StringOrVec::Vec(v) => v,
        StringOrVec::Str(s) => s
            .split(',')
            .map(|x| x.trim().to_string())
            .filter(|x| !x.is_empty())
            .collect(),
    })
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
#[allow(dead_code)]
struct Config {
    project: Project,
    settings: Settings,
    dependencies: BTreeMap<String, Dependency>,
    post_build: Vec<PostBuild>,
    workspace: Option<Workspace>,
    #[serde(skip)]
    verbose: bool,
}

impl Config {
    pub fn new(project_name: &str) -> Self {
        let mut config = Config::default();
        config.project.name = Some(project_name.to_string());
        config
    }

    pub fn load() -> Result<Self> {
        let working_directory = std::env::current_dir()?;
        let config_file = Self::find_config_file(&working_directory)?;
        let contents = std::fs::read_to_string(config_file)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Walk upwards till we find the first `config.toml`
    fn find_config_file(start: &std::path::Path) -> Result<std::path::PathBuf> {
        let mut dir = Some(start);
        while let Some(d) = dir {
            let candidate = d.join("config.toml");
            if candidate.is_file() {
                return Ok(candidate);
            }
            dir = d.parent();
        }
        Err(Error::NoConfig())
    }
}

#[derive(Clone, Debug)]
struct Arguments {
    command: String,
    config: Config,
    file: Option<String>,
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
                return Err(Error::Arguments(
                    "Project name is required for `new` command".to_string(),
                ));
            }
            let project_name = args[2].clone();
            Config::new(&project_name)
        }
        "help" | "version" => Config::default(),
        _ => return Err(Error::Arguments("Unknown command".to_string())),
    };

    config.verbose = args.contains(&"--verbose".to_string()) || args.contains(&"-v".to_string());

    let file = if command == "run" && args.len() > 2 && !args[2].starts_with('-') {
        Some(args[2].clone())
    } else {
        None
    };

    Ok(Arguments {
        command: command.clone(),
        config,
        file,
    })
}

fn create_source_file(file_path: &PathBuf) -> Result<()> {
    if file_path.exists() {
        return Err(Error::ProjectCreation(format!(
            "File {} already exists",
            file_path.display()
        )));
    }

    let file_ext = file_path
        .extension()
        .and_then(|os_str| os_str.to_str())
        .unwrap_or("c");
    let is_cpp = file_ext == "cpp";

    let content = if is_cpp {
        r#"#include <iostream>

int main(int argc, char** argv) {
  std::cout << "Hello, World!" << std::endl;
  return 0;
}
"#
    } else {
        r#"#include <stdio.h>

int main(int argc, char** argv) {
  printf("Hello, World!\n");
  return 0;
}
"#
    };

    std::fs::write(file_path, content)?;
    println!("Created file: {}", file_path.display());

    Ok(())
}

fn create_new_project(name: &str) -> Result<()> {
    let path = std::path::Path::new(name);
    if name.ends_with(".c") || name.ends_with(".cpp") {
        return create_new_module(name);
    }
    if path.exists() {
        return Err(Error::ProjectCreation(
            "Project directory already exists".to_string(),
        ));
    }
    std::fs::create_dir(path)?;

    for dir in &["src", "bin", "lib"] {
        std::fs::create_dir(path.join(dir))?;
    }

    // @TODO: file type based on project (c/c++)
    let main_file_path = path.join("src").join("main.c");
    create_source_file(&main_file_path)?;

    let config_file_path = path.join("config.toml");
    let mut config_file = std::fs::File::create(config_file_path)?;
    writeln!(config_file, "[project]\nname = \"{}\"\n\n[settings]\nlanguage = \"c\"\nstandard = \"c99\"\ncompiler = \"gcc\"\ntype = \"bin\"\ntarget = \"x86_64\"\nmode = \"debug\"", name)?;

    let gitignore_path = path.join(".gitignore");
    let mut gitignore_file = std::fs::File::create(gitignore_path)?;
    writeln!(gitignore_file, "/bin\n*.o\n*.a\n*.so\n*.dll")?;

    println!("Created project: {}", name);

    Ok(())
}

fn create_new_module(module_name: &str) -> Result<()> {
    let module_path = PathBuf::from(module_name);
    if module_path.exists() {
        return Err(Error::ProjectCreation(format!(
            "File '{}' already exists",
            module_name
        )));
    }
    create_source_file(&module_path)
}

fn manage_dependencies(config: &Config) -> Result<()> {
    log(config, "Managing dependencies");

    let current_dir = std::env::current_dir()?;
    let project_lib_path = current_dir.join("lib");
    std::fs::create_dir_all(&project_lib_path)?;

    let global_lib_path = shellexpand::tilde(GLOBAL_LIB_PATH);

    for lib in &config.settings.libraries {
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
            return Err(Error::Library(format!(
                "Library {} not found in global library path",
                lib
            )));
        }
    }

    Ok(())
}

fn collect_sources(root: &std::path::Path) -> Result<Vec<PathBuf>> {
    const SOURCE_EXTS: [&str; 5] = ["c", "cpp", "cc", "cxx", "c++"];
    let mut stack = vec![root.to_path_buf()];
    let mut visited = std::collections::HashSet::new();
    let mut sources = Vec::new();

    while let Some(dir) = stack.pop() {
        if let Ok(canonical) = dir.canonicalize() {
            if !visited.insert(canonical) {
                continue;
            }
        }
        for entry in std::fs::read_dir(&dir)?.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| SOURCE_EXTS.contains(&e.to_ascii_lowercase().as_str()))
                .unwrap_or(false)
            {
                sources.push(path);
            }
        }
    }

    sources.sort();
    Ok(sources)
}

fn cc_driver(settings: &Settings) -> &'static str {
    match (&settings.compiler, &settings.language) {
        (Compiler::GCC, Language::CPP) => "g++",
        (Compiler::GCC, _) => "gcc",
        (Compiler::CLANG, Language::CPP) => "clang++",
        (Compiler::CLANG, _) => "clang",
        (Compiler::MSVC, _) => "cl.exe", // unused for the MSVC path
    }
}

fn cc_std_flag(standard: &Standard) -> &'static str {
    match standard {
        Standard::C89 => "-std=c89",
        Standard::C99 => "-std=c99",
        Standard::C11 => "-std=c11",
        Standard::C17 => "-std=c17",
        Standard::CPP98 => "-std=c++98",
        Standard::CPP11 => "-std=c++11",
        Standard::CPP14 => "-std=c++14",
        Standard::CPP17 => "-std=c++17",
        Standard::CPP20 => "-std=c++20",
        Standard::CPP23 => "-std=c++23",
    }
}

fn msvc_std_flag(standard: &Standard) -> Option<&'static str> {
    match standard {
        Standard::C89 => None,
        Standard::C99 | Standard::C11 => Some("/std:c11"),
        Standard::C17 => Some("/std:c17"),
        Standard::CPP98 | Standard::CPP11 | Standard::CPP14 => Some("/std:c++14"),
        Standard::CPP17 => Some("/std:c++17"),
        Standard::CPP20 => Some("/std:c++20"),
        Standard::CPP23 => Some("/std:c++latest"),
    }
}

fn cc_compile_flags(settings: &Settings) -> Vec<String> {
    let mut flags = vec![cc_std_flag(&settings.standard).to_string()];
    match settings.mode {
        Mode::Debug => flags.push("-g".to_string()),
        Mode::Release => {
            flags.push("-O3".to_string());
            flags.push("-DNDEBUG".to_string());
        }
    }
    if settings.target == Target::X86_64 {
        flags.push("-m64".to_string());
    }
    for d in &settings.defines {
        flags.push(format!("-D{d}"));
    }
    flags.extend(settings.cflags.iter().cloned());
    flags
}

fn cc_link_tail(settings: &Settings) -> Vec<String> {
    let mut tail = Vec::new();
    if let Mode::Release = settings.mode {
        tail.push("-s".to_string());
    }
    tail.extend(settings.lflags.iter().cloned());
    for lib in &settings.system_libs {
        tail.push(format!("-l{lib}"));
    }
    tail
}

fn msvc_compile_flags(settings: &Settings) -> Vec<String> {
    let mut flags = Vec::new();
    if let Some(std) = msvc_std_flag(&settings.standard) {
        flags.push(std.to_string());
    }
    if let Language::CPP = settings.language {
        flags.push("/EHsc".to_string());
    }
    match settings.mode {
        Mode::Debug => flags.push("/Zi".to_string()),
        Mode::Release => {
            flags.push("/O2".to_string());
            flags.push("/DNDEBUG".to_string());
        }
    }
    for d in &settings.defines {
        flags.push(format!("/D{d}"));
    }
    flags.extend(settings.cflags.iter().cloned());
    flags
}

fn msvc_link_tail(settings: &Settings) -> Vec<String> {
    let mut tail = Vec::new();
    if settings.target == Target::X86_64 {
        tail.push("/MACHINE:X64".to_string());
    }
    tail.extend(settings.lflags.iter().cloned());
    for lib in &settings.system_libs {
        if lib.to_ascii_lowercase().ends_with(".lib") {
            tail.push(lib.clone());
        } else {
            tail.push(format!("{lib}.lib"));
        }
    }
    if tail.is_empty() {
        return tail;
    }
    let mut out = vec!["/link".to_string()];
    out.extend(tail);
    out
}

fn build_project(config: Config) -> Result<()> {
    log(&config, "Starting build process");
    manage_dependencies(&config)?;

    let current_dir = std::env::current_dir()?;
    let src_path = current_dir.join("src");
    let lib_path = current_dir.join("lib");
    let bin_path = current_dir.join("bin");
    std::fs::create_dir_all(&bin_path)?;

    let project_name = config
        .project
        .name
        .as_ref()
        .ok_or_else(|| Error::Config("Project name not found".to_string()))?;
    let output_file = bin_path.join(project_name);

    let source_files = collect_sources(&src_path)?;
    if source_files.is_empty() {
        return Err(Error::Config(
            "No source files found in src directory".to_string(),
        ));
    }

    let mut include_dirs: Vec<PathBuf> = vec![src_path.clone(), lib_path.clone()];
    include_dirs.extend(
        config
            .settings
            .include_dirs
            .iter()
            .map(|d| current_dir.join(d)),
    );
    let include_dirs: Vec<PathBuf> = include_dirs.into_iter().filter(|p| p.exists()).collect();

    let mut args = Vec::new();
    let compiler = match config.settings.compiler {
        Compiler::GCC | Compiler::CLANG => {
            let compiler = cc_driver(&config.settings);

            args.push(format!("-o{}", output_file.to_str().unwrap()));
            for inc in &include_dirs {
                args.push(format!("-I{}", inc.to_str().unwrap()));
            }
            args.extend(cc_compile_flags(&config.settings));
            args.extend(
                source_files
                    .iter()
                    .map(|path| path.to_str().unwrap().to_string()),
            );
            args.extend(cc_link_tail(&config.settings));

            compiler
        }
        Compiler::MSVC => {
            let compiler = "cl.exe";
            args.push(format!("/Fe:{}", output_file.to_str().unwrap()));
            for inc in &include_dirs {
                args.push(format!("/I{}", inc.to_str().unwrap()));
            }
            args.extend(msvc_compile_flags(&config.settings));
            args.extend(
                source_files
                    .iter()
                    .map(|path| path.to_str().unwrap().to_string()),
            );
            args.extend(msvc_link_tail(&config.settings));

            compiler
        }
    };

    log(
        &config,
        &format!("Running command: {} {}", compiler, args.join(" ")),
    );

    let output = std::process::Command::new(compiler)
        .args(&args)
        .output()
        .expect("Failed to execute build command");

    if !output.status.success() {
        std::io::stderr().write_all(&output.stderr)?;
        return Err(Error::BuildFailed());
    }

    // @TODO: don't print on `run` mode
    println!("Built `{}`", project_name);
    Ok(())
}

fn run_project(config: &Config) -> Result<()> {
    log(config, "Running project");
    let project_name = config
        .project
        .name
        .as_ref()
        .ok_or_else(|| Error::Config("Project name not found".to_string()))?;
    let current_dir = std::env::current_dir()?;
    let bin_path = current_dir.join("bin").join(project_name);

    if !bin_path.exists() {
        return Err(Error::Config(format!(
            "Binary not found at: {}",
            bin_path.display()
        )));
    }

    log(
        config,
        &format!("Attempting to run: {}", bin_path.display()),
    );

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

fn build_and_run_file(config: &Config, file_name: &str) -> Result<()> {
    log(config, &format!("Building and running file: {}", file_name));

    let temp_dir = PathBuf::from(TEMP_BUILD_DIR);
    std::fs::create_dir_all(&temp_dir)?;

    let source_file = PathBuf::from(file_name);
    let file_stem = source_file.file_stem().unwrap().to_str().unwrap();
    let output_file = temp_dir.join(file_stem);

    let mut args = Vec::new();
    let compiler = match config.settings.compiler {
        Compiler::GCC => "gcc",
        Compiler::CLANG => "clang",
        Compiler::MSVC => "cl.exe",
    };

    match config.settings.compiler {
        Compiler::GCC | Compiler::CLANG => {
            args.push(format!("-o{}", output_file.to_str().unwrap()));
        }
        Compiler::MSVC => {
            args.push(format!("/Fe:{}", output_file.to_str().unwrap()));
        }
    }

    args.push(source_file.to_str().unwrap().to_string());

    log(
        config,
        &format!("Running command: {} {}", compiler, args.join(" ")),
    );

    let output = std::process::Command::new(compiler)
        .args(&args)
        .output()
        .expect("Failed to execute build command");

    if !output.status.success() {
        std::io::stderr().write_all(&output.stderr)?;
        return Err(Error::BuildFailed());
    }

    println!("Built file: {}", file_name);

    log(config, &format!("Running: {}", output_file.display()));

    let run_output = std::process::Command::new(&output_file)
        .output()
        .map_err(|e| Error::IO(e))?;

    std::io::stdout().write_all(&run_output.stdout)?;
    std::io::stderr().write_all(&run_output.stderr)?;

    if !run_output.status.success() {
        return Err(Error::RunFailed(run_output.status.code()));
    }

    std::fs::remove_file(output_file)?;

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
    println!("  run [FILE]    Build and run the project or a specific file");
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
        "new" => {
            if let Some(file) = args.file {
                create_new_module(&file)
            } else {
                create_new_project(&args.config.project.name.unwrap())
            }
        }
        "run" => {
            if let Some(file) = args.file {
                build_and_run_file(&args.config, &file)
            } else {
                build_project(args.config.clone()).and_then(|_| run_project(&args.config))
            }
        }
        "clean" => clean_project(),
        "version" => {
            println!("cbuild version {}", VERSION);
            Ok(())
        }
        "help" => {
            print_help();
            Ok(())
        }
        _ => Err(Error::Arguments("Unknown command".to_string())),
    };

    if let Err(e) = result {
        eprintln!("Error: {:?}", e);
        std::process::exit(1);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_tokens_normalize() {
        assert!(matches!(
            Language::try_from("c".to_string()),
            Ok(Language::C)
        ));
        assert!(matches!(
            Language::try_from("cpp".to_string()),
            Ok(Language::CPP)
        ));
        assert!(matches!(
            Language::try_from("C++".to_string()),
            Ok(Language::CPP)
        ));
        assert!(matches!(
            Language::try_from("CXX".to_string()),
            Ok(Language::CPP)
        ));
        assert!(Language::try_from("rust".to_string()).is_err());
    }

    #[test]
    fn standard_tokens_normalize_incl_cpp23_and_legacy() {
        assert!(matches!(
            Standard::try_from("c99".to_string()),
            Ok(Standard::C99)
        ));
        assert!(matches!(
            Standard::try_from("c++23".to_string()),
            Ok(Standard::CPP23)
        ));
        assert!(matches!(
            Standard::try_from("C++23".to_string()),
            Ok(Standard::CPP23)
        ));
        assert!(matches!(
            Standard::try_from("cpp23".to_string()),
            Ok(Standard::CPP23)
        ));
        assert!(matches!(
            Standard::try_from("CPP20".to_string()),
            Ok(Standard::CPP20)
        ));
        assert!(matches!(
            Standard::try_from("CPP98".to_string()),
            Ok(Standard::CPP98)
        ));
    }

    #[test]
    fn compiler_type_target_mode_normalize() {
        assert!(matches!(
            Compiler::try_from("MSVC".to_string()),
            Ok(Compiler::MSVC)
        ));
        assert!(matches!(
            Compiler::try_from("clang++".to_string()),
            Ok(Compiler::CLANG)
        ));
        assert!(matches!(
            Type::try_from("dll".to_string()),
            Ok(Type::DynLibrary)
        ));
        assert!(matches!(
            Type::try_from("dylib".to_string()),
            Ok(Type::DynLibrary)
        ));
        assert!(matches!(
            Target::try_from("x86_64".to_string()),
            Ok(Target::X86_64)
        ));
        assert!(matches!(
            Target::try_from("x64".to_string()),
            Ok(Target::X86_64)
        ));
        assert!(matches!(
            Mode::try_from("Release".to_string()),
            Ok(Mode::Release)
        ));
    }

    #[test]
    fn string_or_vec_accepts_array_and_comma_string() {
        #[derive(Deserialize)]
        struct Holder {
            #[serde(deserialize_with = "string_or_vec")]
            items: Vec<String>,
        }
        let a: Holder = toml::from_str(r#"items = ["json", "doctest"]"#).unwrap();
        assert_eq!(a.items, vec!["json".to_string(), "doctest".to_string()]);
        let b: Holder = toml::from_str(r#"items = "json, doctest""#).unwrap();
        assert_eq!(b.items, vec!["json".to_string(), "doctest".to_string()]);
    }

    #[test]
    fn test_prj_config_parses_to_same_settings() {
        let toml = include_str!("../tests/test_prj/config.toml");
        let cfg: Config = toml::from_str(toml).unwrap();
        assert_eq!(cfg.project.name.as_deref(), Some("test_prj"));
        assert!(matches!(cfg.settings.language, Language::C));
        assert!(matches!(cfg.settings.standard, Standard::C99));
        assert!(matches!(cfg.settings.compiler, Compiler::GCC));
        assert!(matches!(cfg.settings.build_type, Type::Binary));
        assert!(matches!(cfg.settings.mode, Mode::Debug));
        assert_eq!(cfg.settings.target, Target::X86_64);
        assert!(cfg.settings.libraries.is_empty());
    }

    #[test]
    fn full_v2_config_parses() {
        let toml = r#"
[project]
name = "trace_ai"

[settings]
language = "cpp"
standard = "c++23"
compiler = "msvc"
type = "dylib"
target = "x86_64"
mode = "release"
include_dirs = ["../shared"]
defines = ["NOMINMAX", "WIN32_LEAN_AND_MEAN"]
cflags = ["/MD", "/EHsc"]
lflags = ["/DEBUG:FULL"]
system_libs = ["psapi", "kernel32"]

[dependencies]
shared = { path = "../shared" }
"#;
        let cfg: Config = toml::from_str(toml).unwrap();
        assert_eq!(cfg.project.name.as_deref(), Some("trace_ai"));
        assert!(matches!(cfg.settings.language, Language::CPP));
        assert!(matches!(cfg.settings.standard, Standard::CPP23));
        assert!(matches!(cfg.settings.build_type, Type::DynLibrary));
        assert_eq!(
            cfg.settings.defines,
            vec!["NOMINMAX", "WIN32_LEAN_AND_MEAN"]
        );
        assert_eq!(cfg.settings.cflags, vec!["/MD", "/EHsc"]);
        assert_eq!(cfg.settings.system_libs, vec!["psapi", "kernel32"]);
        assert_eq!(
            cfg.dependencies["shared"].path.as_deref(),
            Some("../shared")
        );
    }

    fn settings(mutate: impl FnOnce(&mut Settings)) -> Settings {
        let mut s = Settings::default();
        mutate(&mut s);
        s
    }

    #[test]
    fn msvc_standard_flags_are_corrected() {
        assert_eq!(msvc_std_flag(&Standard::C89), None);
        assert_eq!(msvc_std_flag(&Standard::C99), Some("/std:c11"));
        assert_eq!(msvc_std_flag(&Standard::C17), Some("/std:c17"));
        assert_eq!(msvc_std_flag(&Standard::CPP17), Some("/std:c++17"));
        assert_eq!(msvc_std_flag(&Standard::CPP20), Some("/std:c++20"));
        assert_eq!(msvc_std_flag(&Standard::CPP23), Some("/std:c++latest"));
    }

    #[test]
    fn msvc_cpp_gets_ehsc_and_defines() {
        let s = settings(|s| {
            s.language = Language::CPP;
            s.standard = Standard::CPP23;
            s.defines = vec!["NOMINMAX".into()];
            s.cflags = vec!["/MD".into()];
        });
        let flags = msvc_compile_flags(&s);
        assert!(flags.contains(&"/std:c++latest".to_string()));
        assert!(flags.contains(&"/EHsc".to_string()));
        assert!(flags.contains(&"/DNOMINMAX".to_string()));
        assert!(flags.contains(&"/MD".to_string()));
    }

    #[test]
    fn msvc_link_tail_places_machine_and_syslibs_after_link() {
        let s = settings(|s| {
            s.system_libs = vec!["psapi".into(), "kernel32.lib".into()];
            s.lflags = vec!["/DEBUG:FULL".into()];
        });
        let tail = msvc_link_tail(&s);
        assert_eq!(tail[0], "/link");
        assert!(tail.contains(&"/MACHINE:X64".to_string()));
        assert!(tail.contains(&"/DEBUG:FULL".to_string()));
        assert!(tail.contains(&"psapi.lib".to_string()));
        assert!(tail.contains(&"kernel32.lib".to_string()));
        assert!(!tail.iter().any(|f| f == "kernel32.lib.lib"));
    }

    #[test]
    fn cc_release_defines_ndebug_and_strips_at_link() {
        let s = settings(|s| {
            s.compiler = Compiler::CLANG;
            s.mode = Mode::Release;
            s.system_libs = vec!["psapi".into()];
        });
        let compile = cc_compile_flags(&s);
        assert!(compile.contains(&"-O3".to_string()));
        assert!(compile.contains(&"-DNDEBUG".to_string()));
        assert!(!compile.contains(&"-s".to_string()));
        let tail = cc_link_tail(&s);
        assert!(tail.contains(&"-s".to_string()));
        assert!(tail.contains(&"-lpsapi".to_string()));
    }

    #[test]
    fn cc_driver_selects_cpp_frontend() {
        assert_eq!(
            cc_driver(&settings(|s| {
                s.compiler = Compiler::CLANG;
                s.language = Language::CPP;
            })),
            "clang++"
        );
        assert_eq!(
            cc_driver(&settings(|s| {
                s.compiler = Compiler::GCC;
                s.language = Language::C;
            })),
            "gcc"
        );
    }
}
