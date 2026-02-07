use std::path::Path;

const ICONS_PATH: &str = "assets/icons";

pub fn get_file_icon_path(filename: &str) -> String {
    let filename_lower = filename.to_lowercase();

    // First we can check the exact filenames and if there are no matches
    // We can move on to the expanded version
    // If there are no matches yet, then we can just leave it as an unrecognizable file format 
    
    let icon_name = match filename_lower.as_str() {
        "dockerfile" | ".dockerignore" => "docker",
        ".gitignore" | ".gitattributes" | ".gitmodules" => "git",
        "package.json" | "package-lock.json" => "npm",
        "cargo.toml" | "cargo.lock" => "rust",
        "tsconfig.json" => "typescript",
        "webpack.config.js" | "webpack.config.ts" => "webpack",
        "gulpfile.js" => "gulp",
        "gruntfile.js" => "grunt", // wtf is a gruntfile 
        ".eslintrc" | ".eslintrc.js" | ".eslintrc.json" => "eslint",
        ".prettierrc" | ".prettierrc.js" | ".prettierrc.json" => "prettier",
        "yarn.lock" => "yarn",
        "license" | "license.md" | "license.txt" => "certificate",
        "readme.md" | "readme.txt" => "markdown",
        ".editorconfig" => "editorconfig",
        "makefile" => "settings",
        ".env" | ".env.local" | ".env.development" => "dotenv",
        "contributing.md" => "contributing",
        "jest.config.js" | "jest.config.ts" => "jest",
        "cypress.json" => "cypress",
        "babel.config.js" | ".babelrc" => "babel",
        "rollup.config.js" => "rollup",
        "vite.config.js" | "vite.config.ts" => "vitejs",
        "turbo.json" => "turborepo",
        "pnpm-lock.yaml" => "pnpm",
        "firebase.json" => "firebase",
        "next.config.js" => "nextjs",
        "nuxt.config.js" => "nuget",
        "tailwind.config.js" => "tailwind",
        "prisma.schema" => "prisma",
        _ => {
            // check for the file extension and then assign accordingly
            if let Some(ext) = Path::new(filename).extension().and_then(|e| e.to_str()) {
                match ext.to_lowercase().as_str() {
                    "rs" => "rust",
                    "js" | "mjs" | "cjs" => "javascript",
                    "jsx" => "react",
                    "ts" => "typescript",
                    "tsx" => "react", // or "react-alt" if you prefer
                    "py" => "python",
                    "toml" => "toml",
                    "json" => "json",
                    "json5" => "json5",
                    "md" => "markdown",
                    "mdx" => "mdx",
                    "html" | "htm" => "html",
                    "css" => "css",
                    "scss" => "sass",
                    "sass" => "sass",
                    "less" => "less",
                    "java" => "java",
                    "cpp" | "cc" | "cxx" | "c++" => "cpp",
                    "c" => "c",
                    "h" | "hpp" => "cpp-h",
                    "cs" => "csharp",
                    "go" => "go",
                    "rb" => "ruby",
                    "php" => "php",
                    "sql" => "database",
                    "sh" | "bash" | "zsh" => "shell",
                    "yml" | "yaml" => "yaml",
                    "xml" => "markup",
                    "svg" => "svg",
                    "vue" => "vue",
                    "kt" | "kts" => "kotlin",
                    "swift" => "swift",
                    "dart" => "dart",
                    "r" => "rlang",
                    "lua" => "lua",
                    "ex" | "exs" => "elixir",
                    "elm" => "elm",
                    "pl" | "pm" => "perl",
                    "clj" | "cljs" => "clojure",
                    "scala" => "scala",
                    "hs" => "haskell",
                    "coffee" => "coffeescript",
                    "styl" => "stylus",
                    "pug" | "jade" => "pug",
                    "ejs" => "ejs",
                    "hbs" | "handlebars" => "handlebars",
                    "tex" => "tex",
                    "pdf" => "pdf",
                    "zip" | "tar" | "gz" | "rar" | "7z" => "zip",
                    "png" | "jpg" | "jpeg" | "gif" | "webp" | "ico" | "bmp" => "image",
                    "mp4" | "mov" | "avi" | "mkv" | "webm" => "video",
                    "mp3" | "wav" | "flac" | "ogg" => "audio",
                    "log" => "log",
                    "env" => "dotenv",
                    "txt" => "notepad",
                    "wasm" => "wasm",
                    "prisma" => "prisma",
                    "graphql" | "gql" => "graphql",
                    "svelte" => "svelte",
                    "vim" => "vim",
                    "lock" => "key",
                    "gitignore" => "git",
                    "docker" => "docker",
                    "dockerfile" => "docker",
                    "nginx" => "nginx",
                    "cmake" => "cmake",
                    "makefile" => "settings",
                    "gradle" => "gradle",
                    "bat" | "cmd" => "powershell",
                    "ps1" => "powershell",
                    "fish" => "fish",
                    "zig" => "zig",
                    "nim" => "nim",
                    "v" | "vh" => "verilog",
                    "vhd" | "vhdl" => "verilog-sys",
                    "erl" => "erlang",
                    "fs" | "fsx" => "fsharp",
                    "d" => "dlang",
                    "pas" => "pascal",
                    "xaml" => "xaml",
                    "csproj" | "fsproj" | "vbproj" => "visualstudio",
                    "sln" => "visualstudio",
                    "proto" => "proto",
                    "tf" => "terraform",
                    "blade" => "blade",
                    "twig" => "twig",
                    "liquid" => "liquid",
                    "nunjucks" | "njk" => "nunjucks",
                    "jinja" | "j2" => "jinja",
                    "slim" => "slim",
                    "haml" => "haml",
                    "plist" => "settings",
                    "ini" => "settings",
                    "cfg" => "settings",
                    "conf" => "settings",
                    "exe" => "exe",
                    "dll" => "binary",
                    "so" => "binary",
                    _ => "default",
                }
            } else {
                "default"
            }
        }
    };

    let extension = match icon_name {
        "actionscript" | "audio" | "axure" | "binary" | "buckle" | "c" | "c-h" |
        "cake" | "certificate" | "cfc" | "cfm" | "coldfusion" | "cpp-h" |
        "contributing" | "cursor" | "d" | "default" | "denizen" | "dlang" |
        "docz" | "dust" | "editorconfig" | "exe" | "file" | "fish" | "foxpro" |
        "gherkin" | "graphviz" | "haml" | "icon" | "image" | "markojs" |
        "mustache" | "nextflow" | "notepad" | "oso" | "paket" | "pascal" |
        "pascal-project" | "perl" | "png" | "powershell" | "razor" | "rlang" |
        "rst" | "server" | "serverless" | "shell" | "smarty" | "stylelint" |
        "tcl" | "textile" | "toml" | "twig" | "url" | "video" | "xaml" => "png",
        _ => "svg",
    };

    format!("{}/{}.{}", ICONS_PATH, icon_name, extension)
}

pub fn get_folder_icon_path(folder_name: &str, is_open: bool) -> String {
    let folder_lower = folder_name.to_lowercase();

    let folder_icon = match folder_lower.as_str() {
        "src" | "source" => if is_open { "src-open" } else { "src" },
        "test" | "tests" | "__tests__" => if is_open { "tests-open" } else { "tests" },
        "node_modules" => if is_open { "node-open" } else { "node" },
        ".git" => if is_open { "git-open" } else { "git" },
        "public" | "static" | "assets" => if is_open { "images-open" } else { "images" },
        "dist" | "build" | "out" => if is_open { "build-open" } else { "build" },
        "bower_components" => if is_open { "bower-open" } else { "bower" },
        "app" => if is_open { "app-open" } else { "app" },
        "components" | "views" => if is_open { "views-open" } else { "views" },
        "services" => if is_open { "services-open" } else { "services" },
        "styles" | "css" | "scss" => if is_open { "styles-open" } else { "styles" },
        "i18n" | "locales" => if is_open { "i18n-open" } else { "i18n" },
        "db" | "database" => if is_open { "db-open" } else { "db" },
        "cypress" => if is_open { "cypress-open" } else { "cypress" },
        "jest" | "__jest__" => if is_open { "jest-open" } else { "jest" },
        "next" | ".next" => if is_open { "next-open" } else { "next" },
        "bench" | "benchmarks" => if is_open { "bench-open" } else { "bench" },
        "layout" | "layouts" => if is_open { "layout-open" } else { "layout" },
        "theme" | "themes" => if is_open { "theme-open" } else { "theme" },
        ".vscode" => if is_open { "vsc-open" } else { "vsc" },
        _ => if is_open { "default-open" } else { "default" },
    };

    format!("{}/folders/{}.svg", ICONS_PATH, folder_icon)
}
