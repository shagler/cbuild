
#include <stdlib.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>

#include <getopt.h>
#include <sys/stat.h>
#include <sys/types.h>

typedef enum Language {
    C,
    CPP,
} Language;

typedef enum Standard {
    C89,
    C99,
    C11,
    C17,
    CPP98,
    CPP11,
    CPP14,
    CPP17,
    CPP20,
} Standard;

typedef struct Library {
    char* name;
    char* version;
} Library;

typedef enum Build_Compiler {
    GCC,   /* linux default (--gcc) */
    CLANG, /* --clang */
    MSVC,  /* windows default (--msvc) */
} Build_Compiler;

typedef enum Build_Type {
    EXECUTABLE, /* default (--bin) */
    LIBRARY,    /* --lib */
    DYNLIBRARY, /* --dylib */
} Build_Type;

typedef enum Build_Target {
    X86_64_WINDOWS_MSVC,
} Build_Target;

typedef enum Build_Mode {
    DEBUG,   /* default (--debug) */
    RELEASE, /* debug symbols removed & optimization enabled (--release) */
} Build_Mode;

typedef struct Build_Config {
    Language language;
    Standard standard;
    Build_Compiler compiler;
    Build_Type type;
    Build_Target target;

    char* compile_flags;
    char* link_flags;
    bool use_libc;
} Build_Config;

typedef struct Toml_File {
    Build_Config build_config;
    Library* libraries;
} Toml_File;

static bool parse_toml_file(Toml_File* toml_file) {

    return true;
}

typedef struct Arguments {
    char* command;
    char* project_name;
    Build_Config build_config;
} Arguments;

static void print_usage() {
    printf("Usage: cbuild <command> <options>\n");
    printf("Commands:\n");
    printf("  build \t Build the project\n");
    printf("  new <name> \t Create a new project with the given name\n");
    printf("Options:\n");
    printf("  --language <language> \t Set the programming language (c or cpp)\n");
    printf("  --standard <standard> \t Set the language standard (c89, c++11, etc)\n");
    printf("  --compiler <compiler> \t Set the compiler (gcc, clang, or msvc)\n");
    printf("  --type <type> \t\t Set the build type (bin, lib, or dylib)\n");
    printf("  --mode <mode> \t\t Set the build mode (debug or release)\n");
    printf("  --target <target> \t\t Set the build target\n");
    printf("  --compile-flags <flags> \t Set additional compile flags\n");
    printf("  --link-flags <flags> \t\t Set additional link flags\n");
    printf("  --use-libc <bool> \t\t Use libc (true or false)\n");
}

static void parse_command_line_args(int argc, char** argv, Arguments* args) {
    // @TODO: Use our own library, we don't want to use `getopt`
    static struct option long_options[] = {
        {"language", required_argument, 0, 0},
        {"standard", required_argument, 0, 0},
        {"compiler", required_argument, 0, 0},
        {"type", required_argument, 0, 0},
        {"mode", required_argument, 0, 0},
        {"target", required_argument, 0, 0},
        {"compile-flags", required_argument, 0, 0},
        {"link-flags", required_argument, 0, 0},
        {"use-libc", required_argument, 0, 0},
        {0, 0, 0, 0}
    };

    if (argc < 2) {
        print_usage();
        exit(EXIT_FAILURE);
    }

    args->command = argv[1];
    if (strcmp(args->command, "new") == 0 && argv >= 3) {
        args->project_name = argv[2];
    }

    int opt = 0;
    int long_index = 0;
    while ((opt = getopt_long(argc, argv, ":", long_options, &long_index)) != -1) {
        switch (opt) {
            case 0: {
                        if (strcmp(long_options[long_index].name, "language") == 0) {
                            args->build_config.language = (strcmp(optarg, "cpp") == 0) ? CPP : C;
                        }
                        else if (strcmp(long_options[long_index].name, "standard") == 0) {
                            // @TODO:
                        }
                        else if (strcmp(long_options[long_index].name, "compiler") == 0) {
                            // @TODO:
                        }
                        else if (strcmp(long_options[long_index].name, "type") == 0) {
                            // @TODO:
                        }
                        else if (strcmp(long_options[long_index].name, "mode") == 0) {
                            // @TODO:
                        }
                        else if (strcmp(long_options[long_index].name, "target") == 0) {
                            // @TODO:
                        }
                        else if (strcmp(long_options[long_index].name, "compile-flags") == 0) {
                            args->build_config.compile_flags = optarg;
                        }
                        else if (strcmp(long_options[long_index].name, "link-flags") == 0) {
                            args->build_config.link_flags = optarg;
                        }
                        else if (strcmp(long_options[long_index].name, "use-libc") == 0) {
                            args->build_config.use_libc = (strcmp(optarg, "true") == 0) ? true : false;
                        }
                    } break;
        }
    }
}

static bool create_new_project(const char* project_name) {
    /* create the project directory */
    int mkdir_result = mkdir(project_name, S_IRWXU | S_IRWXG | S_IROTH | S_IXOTH);
    if (mkdir_result != 0) {
        fprintf(stderr, "Error: Unable to create project directory '%s'\n", project_name);
        return false;
    }

    /* create the src, lib, and bin directories */
    char src_path[256], lib_path[256], bin_path[256];
    snprintf(src_path, sizeof(src_path), "%s/src", project_name);
    snprintf(lib_path, sizeof(lib_path), "%s/lib", project_name);
    snprintf(bin_path, sizeof(bin_path), "%s/bin", project_name);
    mkdir(src_path, S_IRWXU | S_IRWXG | S_IROTH | S_IXOTH);
    mkdir(lib_path, S_IRWXU | S_IRWXG | S_IROTH | S_IXOTH);
    mkdir(bin_path, S_IRWXU | S_IRWXG | S_IROTH | S_IXOTH);

    /* create a default `build.toml` file in the project directory */
    char build_toml_path[256];
    snprintf(build_toml_path, sizeof(build_toml_path), "%s/build.toml", project_name);
    FILE* build_toml_file = fopen(build_toml_path);
    if (build_toml_file == NULL) {
        fprintf(stderr, "Error: Unable to create `build.toml` file\n");
        return false;
    }

    /* write the default `build.toml` content */
    const char* default_build_toml_content =
        "[project]\n"
        "name = \"%s\"\n\n"
        "[build]\n"
        "language = \"c\"\n"
        "standard = \"c89\"\n"
        "compiler = \"gcc\"\n"
        "type = \"bin\"\n"
        "target = \"x86_64-windows-msvc\"\n"
        "mode = \"debug\"\n"
        "compile_flags = [\"-Wall\", \"-Wextra\", \"-Werror\", \"-pedantic\"]\n"
        "link_flags = [\"lm\"]\n"
        "use_libc = true\n\n"
        "[libraries]\n"
        "core = \"1.0.0\"\n";

    fprintf(build_toml_file, default_build_toml_content, project_name);
    fclose(build_toml_file);

    printf("Project '%s' successfully created\n", project_name);

    return true;
}

static void compile_source_files(Toml_File* toml_file) {
    /* get the list of source files using `dirent.h` 
     * to iterate through the `/src` directory */
    char* source_file_path = "";
    char* object_file_path = "";

    /* for each source file, construct the compilation command */
    char compile_command[1024];
    snprintf(compile_command, sizeof(compile_command), "%s -std=%s %s -c %s -o %s",
            toml_file->build_config.compiler,
            toml_file->build_config.standard,
            toml_file->build_config.compile_flags,
            source_file_path,
            object_file_path);

    /* execute the compilation command */
    int compile_result = system(compile_command);
    if (compile_result != 0) {
        fprintf(stderr, "Error: Compilation of '%s' failed\n", source_file_path);
    }
}

static void link_object_files(Toml_File* toml_file) {
    /* get the list of object files using `dirent.h`
     * to iterate through the `/src` directory */
    char* output_executable_path = "";
    char* object_files = "";

    /* construct linking command */
    char link_command[1024] = {0};
    snprintf(link_command, sizeof(link_command),
            "%s -o %s %s %s",
            toml_file->build_config.compiler,
            output_executable_path,
            object_files,
            toml_file->build_config.link_flags);

    /* execute the linking command */
    int link_result = system(link_command);
    if (link_result != 0) {
        fprintf(stderr, "Error: Linking failed\n");
    }
}

static void link_object_files(Toml_File* toml_file) {
	if (toml_file->build_config.mode == DEBUG) {
		strcat(toml_file->build_config.compile_flags, "-g");
	}
	else if (toml_file->build_config.mode == RELEASE) {
		strcat(toml_file->build_config.compile_flags, "-O2");
	}
}

int main(int argc, char** argv) {
	Arguments args = {0};
	parse_command_line_args(argc, argv, &args);

	if (strcmp(args.command, "build") == 0) {
		Toml_File toml_file = {0};
		if (!parse_toml_file(&toml_file)) {
			fprintf(stderr, "Error: Failed to parse `build.toml` file\n");
			exit(EXIT_FAILURE);
		}

		set_build_mode_flags(&toml_file);
        compile_source_files(&toml_file);
        link_object_files(&toml_file);
	}
	else if (strcmp(args.command, "new") == 0) {
		if (argc < 3) {
			printf("Usage: cbuild new <project_name>\n");
			exit(EXIT_FAILURE);
		}
		create_new_project(args.project_name);
	}
	else {
		print_usage();
		exit(EXIT_FAILURE);
	}

	return 0;
}
