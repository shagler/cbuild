
#include <stdlib.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>

#include <getopt.h>
#include <dirent.h>
#include <sys/stat.h>
#include <sys/types.h>

/* supported programming languages */
typedef enum Language {
    C,
    CPP,
} Language;

/* supported language standards */
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

/* library dependency */
typedef struct Library {
    char* name;
    char* version;
} Library;

/* supported compilers */
typedef enum Build_Compiler {
    GCC,   /* linux default (--gcc) */
    CLANG, /* --clang */
    MSVC,  /* windows default (--msvc) */
} Build_Compiler;

/* supported build types */
typedef enum Build_Type {
    EXECUTABLE, /* default (--bin) */
    LIBRARY,    /* --lib */
    DYNLIBRARY, /* --dylib */
} Build_Type;

/* supported build targets */
typedef enum Build_Target {
    X86_64_WINDOWS_MSVC,
} Build_Target;

/* supported build modes */
typedef enum Build_Mode {
    DEBUG,   /* default (--debug) */
    RELEASE, /* debug symbols removed & optimization enabled (--release) */
} Build_Mode;

/* build configuration */
typedef struct Build_Config {
    Language language;
    Standard standard;
    Build_Compiler compiler;
    Build_Type type;
    Build_Target target;
	Build_Mode mode;
    char* compile_flags;
    char* link_flags;
    bool use_libc;
} Build_Config;

/* `build.toml` file */
typedef struct Toml_File {
    Build_Config build_config;
    Library* libraries;
} Toml_File;

/* parse a key-value pair from a given line */
static bool parse_key_value_pair(const char* line, char* key, char* value) {
	/* use sscanf to parse the line for a key-value pair seperated by an '=' sign */
	if (sscanf(line, "%[^=]=%s", key, value) != 2) {
		return false;
	}
	return true;
}

/* parse a section from a given line */
static bool parse_section(const char* line, char* section) {
	/* use sscanf to match a section patten in the line */
	if (sscanf(line, "[%[^]]]", section) != 1) {
		return false;
	}
	return true;
}

/* populate the TOML file struture based on the section, key, value */
static void populate_toml_structure(Toml_File* toml_file, const char* section, 
		const char* key, const char* value) {
	if (strcmp(section, "build") == 0) {
		Build_Config* config = &(toml_file->build_config);

		if (strcmp(key, "language") == 0) {
			config->language = (strcmp(value, "cpp") == 0) ? CPP : C;
		}
		else if (strcmp(key, "standard") == 0) {
			/* @TODO: store the standard value into the config structure */
		}
		else if (strcmp(key, "compiler") == 0) {
			/* @TODO: store the compiler value into the config structure */
		}
		else if (strcmp(key, "compile-flags") == 0) {
			config->compile_flags = strdup(value);
		}
		else if (strcmp(key, "link-flags") == 0) {
			config->link_flags = strdup(value);
		}
		else if (strcmp(key, "use-libc") == 0) {
			config->use_libc = (strcmp(value, "true") == 0) ? true : false;
		}
	}
	else if (strcmp(section, "libraries") == 0) {
		/* populate the libraries structure */
	}
}

/* parse a TOML file and populate the Toml_File structure */
static bool parse_toml_file(Toml_File* toml_file, const char* file_path) {
	/* open the TOML file */
	FILE* file = fopen(file_path, "r");
	if (!file) {
		fprintf(stderr, "Error: Could not open file '%s'\n", file_path);
		return false;
	}

	/* process each line in the TOML file */
	char line[256] = {0};
	char section[256] = {0};
	while (fgets(line, sizeof(line), file)) {
		if (line[0] == '[') {
			/* parse the section name */
			if (!parse_section(line, section)) {
				fprintf(stderr, "Error: Invalid section syntax: '%s'\n", line);
				fclose(file);
				return false;
			}
		}
		else if (line[0] != '#' && line[0] != '\n') {
			/* parse the key-value pair */
			char key[256] = {0};
			char value[256] = {0};
			if (!parse_key_value_pair(line, key, value)) {
				fprintf(stderr, "Error: Invalid key-value pair syntax: %s\n", line);
				fclose(file);
				return false;
			}

			/* populate the Toml_File structure based on the section, key, and value */
			populate_toml_structure(toml_file, section, key, value);
		}
	}

	/* close the TOML file */
	fclose(file);

    return true;
}

/* command line arguments */
typedef struct Arguments {
    char* command;
    char* project_name;
    Build_Config build_config;
} Arguments;

/* check if a file has a specific extension */
static bool has_extension(const char* filename, const char* extension) {
	const char* dot = strrchr(filename, '.');
	if (!dot || dot == filename) return false;
	return strcmp(dot + 1, extension) == 0;
}

/* get a list of source files in the specified directory */
static char** get_source_files(const char* src_dir, const char* extension, 
		int* file_count) {
	DIR* dir = {0};
	struct dirent* entry = {0};
	int count = 0;

	/* open the directory */
	if ((dir = opendir(src_dir)) == NULL) {
		fprintf(stderr, "Error: Unable to open source directory '%s'\n", src_dir);
		return NULL;
	}

	/* count the number of source files in the directory */
	while ((entry = readdir(dir)) != NULL) {
		if (entry->d_type == DT_REG && has_extension(entry->d_name, extension)) {
			count++;
		}
	}

	/* allocate memory for the list of source files */
	char** source_files = malloc(count * sizeof(char*));
	if (!source_files) {
		fprintf(stderr, "Error: Unable to allocate memory for source files.\n");
		closedir(dir);
		return NULL;
	}

	/* reset the directory and fill the list of source files */
	rewinddir(dir);
	int index = 0;
	while ((entry = readdir(dir)) != NULL) {
		if (entry->d_type == DT_REG && has_extension(entry->d_name, extension)) {
			source_files[index] = malloc(strlen(entry->d_name) + 1);
			if (!source_files[index]) {
				fprintf(stderr, 
						"Error: Unable to allocate memory for source file named '%s'\n", 
						entry->d_name);
				break;
			}
			strcpy(source_files[index], entry->d_name);
			index++;
		}
	}

	closedir(dir);
	*file_count = count;

	return source_files;
}

/* print usage information for the program */
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

/* parse command line arguments and populate the Arguments structure */
static void parse_command_line_args(int argc, char** argv, Arguments* args) {
    /* @TODO: Use our own library, we don't want to use `getopt` */
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
					if (strcmp(optarg, "c89") == 0) {
						args->build_config.standard = C89;
					}
					else if (strcmp(optarg, "c99") == 0) {
						args->build_config.standard = C99;
					}
					else if (strcmp(optarg, "c11") == 0) {
						args->build_config.standard = C11;
					}
					else if (strcmp(optarg, "c17") == 0) {
						args->build_config.standard = C17;
					}
					else if (strcmp(optarg, "cpp98") == 0) {
						args->build_config.standard = CPP98;
					}
					else if (strcmp(optarg, "cpp11") == 0) {
						args->build_config.standard = CPP11;
					}
					else if (strcmp(optarg, "cpp14") == 0) {
						args->build_config.standard = CPP14;
					}
					else if (strcmp(optarg, "cpp17") == 0) {
						args->build_config.standard = CPP17;
					}
					else if (strcmp(optarg, "cpp20") == 0) {
						args->build_config.standard = CPP20;
					}
                }
                else if (strcmp(long_options[long_index].name, "compiler") == 0) {
					if (strcmp(optarg, "gcc") == 0) {
						args->build_config.compiler = GCC;
					}
					else if (strcmp(optarg, "clang") == 0) {
						args->build_config.compiler = CLANG;
					}
					else if (strcmp(optarg, "msvc") == 0) {
						args->build_config.compiler = MSVC;
					}
                }
                else if (strcmp(long_options[long_index].name, "type") == 0) {
					if (strcmp(optarg, "lib") == 0) {
						args->build_config.type = LIBRARY;
					}
					else if (strcmp(optarg, "dylib") == 0) {
						args->build_config.type = DYNLIBRARY;
					}
					else if (strcmp(optarg, "bin")) {
						args->build_config.type = EXECUTABLE;
					}
					else {
						fprintf(stderr, "Error: Invalid build type\n");
						exit(EXIT_SUCCESS);
					}
                }
                else if (strcmp(long_options[long_index].name, "mode") == 0) {
					if (strcmp(optarg, "release") == 0) {
						args->build_config.mode = RELEASE;
					}
					else if (strcmp(optarg, "debug") == 0) {
						args->build_config.mode = DEBUG;
					}
                }
                else if (strcmp(long_options[long_index].name, "target") == 0) {
					if (strcmp(optarg, "x86_64-windows-msvc") == 0) {
						args->build_config.target = X86_64_WINDOWS_MSVC;
					}
                }
                else if (strcmp(long_options[long_index].name, "compile-flags") == 0) {
					args->build_config.compile_flags = optarg;
                }
                else if (strcmp(long_options[long_index].name, "link-flags") == 0) {
					args->build_config.link_flags = optarg;
                }
                else if (strcmp(long_options[long_index].name, "use-libc") == 0) {
					if (strcmp(optarg, "true") == 0) {

					}
					else if (strcmp(optarg, "false") == 0) {

					}
					else {
						fprintf(stderr, "Error: Invalid boolean\n");
						exit(EXIT_SUCCESS);
					}
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

	/* @TODO: support for more than just C */
	/* create a default `main.c` file in the `/src` directory */
    char main_file_path[256];
    snprintf(main_file_path, sizeof(main_file_path), "%s/src/main.c", project_name);
    FILE* main_file = fopen(main_file_path, "w");
    if (main_file == NULL) {
        fprintf(stderr, "Error: Unable to create `/src/main.c` file\n");
        return false;
    }

    /* create a default `build.toml` file in the project directory */
    char build_toml_path[256];
    snprintf(build_toml_path, sizeof(build_toml_path), "%s/build.toml", project_name);
    FILE* build_toml_file = fopen(build_toml_path, "w");
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
	const char* src_dir = "./src";
	const char* extension = "c";
	int file_count = 0;
	char** source_files = get_source_files(src_dir, extension, &file_count);
	if (source_files) {
		for (int i = 0; i < file_count; ++i) {
			char* source_file_path = source_files[i];
			char object_file_path[1024];
			snprintf(object_file_path, sizeof(object_file_path), "%s.o", source_file_path);

			char* compiler = {0};
			char* standard = {0};
			if (toml_file->build_config.compiler == GCC) {
				compiler = "gcc";
			}
			if (toml_file->build_config.standard == C89) {
				standard = "c89";
			}

			/* construct the compilation command */
			char compile_command[1024];
			snprintf(compile_command, sizeof(compile_command), "%s -std=%s %s -c ./src/%s -o ./bin/%s",
					compiler,
					standard,
					"",
					//toml_file->build_config.compile_flags,
					source_file_path,
					object_file_path);

			/* execute the compilation command */
			printf("%s\n", compile_command);
			int compile_result = system(compile_command);
			if (compile_result != 0) {
				fprintf(stderr, "Error: Compilation of '%s' failed\n", source_file_path);
			}

			free(source_files[i]);
		}
		free(source_files);
	}
}

static void link_object_files(Toml_File* toml_file) {
    char* output_executable_path = "./bin/main";
    char* object_files = "./bin/main.c.o";

	char* compiler = {0};
	if (toml_file->build_config.compiler == GCC) {
		compiler = "gcc";
	}

    /* construct linking command */
    char link_command[1024] = {0};
    snprintf(link_command, sizeof(link_command),
            "%s -o %s %s %s",
            compiler,
            output_executable_path,
            object_files,
			"");
            //toml_file->build_config.link_flags);

    /* execute the linking command */
	printf("%s\n", link_command);
    int link_result = system(link_command);
    if (link_result != 0) {
        fprintf(stderr, "Error: Linking failed\n");
    }
}

static void set_build_mode_flags(Toml_File* toml_file) {
	if (toml_file->build_config.mode == DEBUG) {
		//strcat(toml_file->build_config.compile_flags, "-g");
	}
	else if (toml_file->build_config.mode == RELEASE) {
		//strcat(toml_file->build_config.compile_flags, "-O2");
	}
}

int main(int argc, char** argv) {
	Arguments args = {0};
	parse_command_line_args(argc, argv, &args);

	if (strcmp(args.command, "build") == 0) {
		Toml_File toml_file = {0};
		const char* build_toml_path = "./build.toml";
		if (!parse_toml_file(&toml_file, build_toml_path)) {
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
