use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Base directory for file icons (`assets/icons/`)
fn icons_base() -> PathBuf {
    crate::resources::resource_dir().join("assets").join("icons")
}

/// Base directory for folder icons (`assets/icons/folders/`)
fn folder_icons_base() -> PathBuf {
    icons_base().join("folders")
}

/// Resolve an icon name to its full path, trying `.svg` first then `.png`.
fn resolve_icon(base: &Path, name: &str) -> String {
    let svg = base.join(format!("{}.svg", name));
    if svg.exists() {
        return svg.to_string_lossy().into_owned();
    }
    let png = base.join(format!("{}.png", name));
    if png.exists() {
        return png.to_string_lossy().into_owned();
    }
    // Return the svg path as fallback even if missing
    svg.to_string_lossy().into_owned()
}

// ── File extension → icon name ──────────────────────────────────────────────
static FILE_EXT_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    HashMap::from([
        ("rs", "rust"), ("py", "python"), ("js", "javascript"), ("mjs", "javascript"),
        ("cjs", "javascript"), ("ts", "typescript"), ("jsx", "react"), ("tsx", "react"),
        ("html", "html"), ("htm", "html"), ("css", "css"), ("scss", "sass"),
        ("sass", "sass"), ("less", "less"), ("json", "json"), ("json5", "json5"),
        ("yaml", "yaml"), ("yml", "yml"), ("toml", "toml"), ("md", "markdown"),
        ("mdx", "mdx"), ("go", "go"), ("java", "java"), ("c", "c"), ("h", "c-h"),
        ("cpp", "cpp"), ("cc", "cpp"), ("cxx", "cpp"), ("hpp", "cpp-h"),
        ("cs", "csharp"), ("rb", "ruby"), ("swift", "swift"), ("kt", "kotlin"),
        ("kts", "kotlin"), ("dart", "dart"), ("lua", "lua"), ("pl", "perl"),
        ("pm", "perl"), ("php", "php"), ("sql", "database"), ("sh", "shell"),
        ("bash", "shell"), ("zsh", "shell"), ("fish", "fish"), ("ps1", "powershell"),
        ("svg", "svg"), ("xml", "markup"), ("graphql", "graphql"), ("gql", "graphql"),
        ("proto", "proto"), ("ex", "elixir"), ("exs", "elixir"), ("erl", "erlang"),
        ("hs", "haskell"), ("scala", "scala"), ("clj", "clojure"), ("cljs", "clojure"),
        ("nim", "nim"), ("zig", "zig"), ("vue", "vue"), ("svelte", "svelte"),
        ("elm", "elm"), ("rkt", "racket"), ("r", "rlang"), ("jl", "julia"),
        ("tex", "tex"), ("rst", "rst"), ("pdf", "pdf"), ("wasm", "wasm"),
        ("lock", "key"), ("log", "log"), ("env", "dotenv"), ("dockerfile", "docker"),
        ("dockerignore", "docker"), ("gitignore", "git"), ("gitattributes", "git"),
        ("gitmodules", "git"), ("editorconfig", "editorconfig"),
        ("eslintrc", "eslint"), ("prettierrc", "prettier"),
        ("png", "image"), ("jpg", "image"), ("jpeg", "image"), ("gif", "image"),
        ("ico", "image"), ("webp", "image"), ("bmp", "image"),
        ("mp3", "audio"), ("wav", "audio"), ("ogg", "audio"), ("flac", "audio"),
        ("mp4", "video"), ("mov", "video"), ("avi", "video"), ("mkv", "video"),
        ("zip", "zip"), ("tar", "zip"), ("gz", "zip"), ("rar", "zip"),
        ("exe", "exe"), ("bin", "binary"), ("dll", "binary"),
        ("csv", "excel"), ("xlsx", "excel"), ("xls", "excel"),
        ("doc", "word"), ("docx", "word"), ("pptx", "powerpoint"),
        ("nix", "nix"), ("sol", "solidity"), ("tf", "terraform"),
        ("prisma", "prisma"), ("gradle", "gradle"), ("groovy", "groovy"),
        ("f90", "fortran"), ("f95", "fortran"),
        ("d", "dlang"), ("cr", "crystal"), ("fs", "fsharp"), ("fsx", "fsharp"),
        ("ml", "ocaml"), ("mli", "ocaml"),
        ("hx", "haxe"), ("pas", "pascal"), ("pp", "pascal"),
        ("pug", "pug"), ("slim", "slim"), ("haml", "haml"),
        ("styl", "stylus"), ("postcss", "postcss"),
        ("coffee", "coffeescript"), ("litcoffee", "coffeescript"),
        ("asm", "assembly"), ("s", "assembly"),
        ("m", "objectivec"), ("mm", "objectivecpp"),
        ("ipynb", "jupyter"), ("rmd", "rstudio"),
        ("twig", "twig"), ("hbs", "handlebars"), ("mustache", "mustache"),
        ("ejs", "ejs"), ("njk", "nunjucks"), ("jinja", "jinja"), ("jinja2", "jinja"),
    ])
});

// ── Full filename → icon name ───────────────────────────────────────────────
static FILE_NAME_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    HashMap::from([
        ("dockerfile", "docker"), ("docker-compose.yml", "docker"),
        ("docker-compose.yaml", "docker"),
        ("makefile", "gnu"), ("cmakelists.txt", "cmake"),
        (".gitignore", "git"), (".gitattributes", "git"), (".gitmodules", "git"),
        (".env", "dotenv"), (".env.local", "dotenv"), (".env.development", "dotenv"),
        (".env.production", "dotenv"),
        ("package.json", "npm"), ("package-lock.json", "npm"),
        ("cargo.toml", "rust"), ("cargo.lock", "rust"),
        ("tsconfig.json", "typescript"), ("jsconfig.json", "javascript"),
        (".eslintrc", "eslint"), (".eslintrc.js", "eslint"), (".eslintrc.json", "eslint"),
        (".prettierrc", "prettier"), (".prettierrc.js", "prettier"),
        (".prettierrc.json", "prettier"),
        ("jest.config.js", "jest"), ("jest.config.ts", "jest"),
        ("webpack.config.js", "webpack"), ("webpack.config.ts", "webpack"),
        ("rollup.config.js", "rollup"), ("rollup.config.ts", "rollup"),
        ("vite.config.js", "vitejs"), ("vite.config.ts", "vitejs"),
        ("next.config.js", "nextjs"), ("next.config.mjs", "nextjs"),
        ("tailwind.config.js", "tailwind"), ("tailwind.config.ts", "tailwind"),
        ("babel.config.js", "babel"), (".babelrc", "babel"),
        ("license", "certificate"), ("licence", "certificate"),
        ("readme.md", "markdown"), ("changelog.md", "markdown"),
        ("todo.md", "todo"), ("todo.txt", "todo"), ("todo", "todo"),
        (".editorconfig", "editorconfig"),
        ("yarn.lock", "yarn"), (".yarnrc", "yarn"),
        ("pnpm-lock.yaml", "pnpm"), (".npmrc", "npm"),
        ("nginx.conf", "nginx"),
        (".prettierignore", "prettier"), (".eslintignore", "eslint"),
        ("vercel.json", "vercel"), ("netlify.toml", "server"),
        (".dockerignore", "docker"),
    ])
});

// ── Folder name → icon name ─────────────────────────────────────────────────
static FOLDER_NAME_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    HashMap::from([
        ("src", "src"), ("source", "src"), ("lib", "src"),
        ("build", "build"), ("dist", "build"), ("out", "build"), ("output", "build"),
        ("node_modules", "node"),
        ("test", "tests"), ("tests", "tests"), ("__tests__", "tests"), ("spec", "tests"),
        (".git", "git"),
        ("styles", "styles"), ("css", "styles"), ("style", "styles"),
        ("images", "images"), ("img", "images"), ("assets", "images"), ("icons", "images"),
        ("views", "views"), ("pages", "views"), ("screens", "views"),
        ("db", "db"), ("database", "db"), ("migrations", "db"),
        ("i18n", "i18n"), ("locale", "i18n"), ("locales", "i18n"), ("lang", "i18n"),
        ("app", "app"),
        ("theme", "theme"), ("themes", "theme"),
        ("services", "services"), ("service", "services"),
        ("layout", "layout"), ("layouts", "layout"),
        (".vscode", "vsc"),
        ("bench", "bench"), ("benchmarks", "bench"),
        ("cypress", "cypress"),
        ("next", "next"),
        ("bower_components", "bower"),
        ("save", "save"), ("backup", "save"),
    ])
});

pub fn get_file_icon(filename: &str) -> String {
    let filename_lower = filename.to_lowercase();
    let base = icons_base();

    // 1. Try exact filename match
    if let Some(&icon_name) = FILE_NAME_MAP.get(filename_lower.as_str()) {
        return resolve_icon(&base, icon_name);
    }

    // 2. Try compound extension (e.g. "test.spec.ts" → "spec.ts")
    let parts: Vec<&str> = filename_lower.split('.').collect();
    for i in 1..parts.len() {
        let compound_ext = parts[i..].join(".");
        if let Some(&icon_name) = FILE_EXT_MAP.get(compound_ext.as_str()) {
            return resolve_icon(&base, icon_name);
        }
    }

    // 3. Try simple extension
    if let Some(ext) = Path::new(filename).extension().and_then(|e| e.to_str()) {
        if let Some(&icon_name) = FILE_EXT_MAP.get(ext.to_lowercase().as_str()) {
            return resolve_icon(&base, icon_name);
        }
    }

    // 4. Default file icon
    resolve_icon(&base, "file")
}

pub fn get_folder_icon(folder_name: &str, is_open: bool) -> String {
    let folder_lower = folder_name.to_lowercase();
    let base = folder_icons_base();

    let icon_base_name = FOLDER_NAME_MAP
        .get(folder_lower.as_str())
        .copied()
        .unwrap_or("default");

    let name = if is_open {
        format!("{}-open", icon_base_name)
    } else {
        icon_base_name.to_string()
    };

    resolve_icon(&base, &name)
}
