use include_dir::{include_dir, Dir};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use iced::widget::image;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IconFormat {
    Svg,
    Png,
}

#[derive(Clone, Copy, Debug)]
pub struct IconAsset {
    pub format: IconFormat,
    pub bytes: &'static [u8],
}

static ICONS_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/src/assets/icons");
static ICON_HANDLE_CACHE: Lazy<Mutex<HashMap<IconCacheKey, image::Handle>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

const SVG_ICON_RASTER_SIZE: u32 = 64;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct IconCacheKey {
    ptr: usize,
    len: usize,
    size: u32,
}

impl IconCacheKey {
    fn new(bytes: &'static [u8], size: u32) -> Self {
        Self {
            ptr: bytes.as_ptr() as usize,
            len: bytes.len(),
            size,
        }
    }
}

/// Detemine width and height of svg icons to be rendered.
fn rasterize_svg_icon(bytes: &'static [u8], size: u32) -> Option<image::Handle> {
    let options = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data(bytes, &options).ok()?;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(size, size)?;

    resvg::render(
        &tree,
        resvg::usvg::FitTo::Size(size, size),
        resvg::tiny_skia::Transform::default(),
        pixmap.as_mut(),
    )?;

    Some(image::Handle::from_rgba(
        pixmap.width(),
        pixmap.height(),
        pixmap.take(),
    ))
}

/// Detemine width and height of png icons to be rendered.
fn rasterize_png_icon(bytes: &'static [u8], size: u32) -> Option<image::Handle> {
    let image = ::image::load_from_memory(bytes).ok()?.into_rgba8();
    let resized = if image.width() == size && image.height() == size {
        image
    } else {
        ::image::imageops::resize(&image, size, size, ::image::imageops::FilterType::Triangle)
    };

    Some(image::Handle::from_rgba(
        resized.width(),
        resized.height(),
        resized.into_raw(),
    ))
}

pub fn icon_handle(icon: IconAsset, size: u32) -> image::Handle {
    let key = IconCacheKey::new(icon.bytes, size);
    let mut cache = ICON_HANDLE_CACHE.lock().expect("icon cache poisoned");

    if let Some(handle) = cache.get(&key) {
        return handle.clone();
    }

    match icon.format {
        IconFormat::Png => {
            let handle = rasterize_png_icon(icon.bytes, size)
                .unwrap_or_else(|| image::Handle::from_bytes(icon.bytes));
            cache.insert(key, handle.clone());
            handle
        }
        IconFormat::Svg => {
            let handle = rasterize_svg_icon(icon.bytes, size.max(SVG_ICON_RASTER_SIZE))
                .unwrap_or_else(|| image::Handle::from_bytes(icon.bytes));
            cache.insert(key, handle.clone());
            handle
        }
    }
}

fn resolve_icon(base: &str, name: &str) -> IconAsset {
    let svg_path = if base.is_empty() {
        format!("{name}.svg")
    } else {
        format!("{base}/{name}.svg")
    };
    if let Some(svg) = ICONS_DIR.get_file(&svg_path) {
        return IconAsset {
            format: IconFormat::Svg,
            bytes: svg.contents(),
        };
    }

    let png_path = if base.is_empty() {
        format!("{name}.png")
    } else {
        format!("{base}/{name}.png")
    };
    if let Some(png) = ICONS_DIR.get_file(&png_path) {
        return IconAsset {
            format: IconFormat::Png,
            bytes: png.contents(),
        };
    }

    let fallback = ICONS_DIR
        .get_file("file.png")
        .expect("embedded fallback icon src/assets/icons/file.png must exist");
    IconAsset {
        format: IconFormat::Png,
        bytes: fallback.contents(),
    }
}

static FILE_EXT_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    HashMap::from([
        ("rs", "rust"),
        ("py", "python"),
        ("js", "javascript"),
        ("mjs", "javascript"),
        ("cjs", "javascript"),
        ("ts", "typescript"),
        ("jsx", "react"),
        ("tsx", "react"),
        ("html", "html"),
        ("htm", "html"),
        ("css", "css"),
        ("scss", "sass"),
        ("sass", "sass"),
        ("less", "less"),
        ("json", "json"),
        ("json5", "json5"),
        ("yaml", "yaml"),
        ("yml", "yml"),
        ("toml", "toml"),
        ("md", "markdown"),
        ("mdx", "mdx"),
        ("go", "go"),
        ("java", "java"),
        ("c", "c"),
        ("h", "c-h"),
        ("cpp", "cpp"),
        ("cc", "cpp"),
        ("cxx", "cpp"),
        ("hpp", "cpp-h"),
        ("cs", "csharp"),
        ("rb", "ruby"),
        ("swift", "swift"),
        ("kt", "kotlin"),
        ("kts", "kotlin"),
        ("dart", "dart"),
        ("lua", "lua"),
        ("pl", "perl"),
        ("pm", "perl"),
        ("php", "php"),
        ("sql", "database"),
        ("sh", "shell"),
        ("bash", "shell"),
        ("zsh", "shell"),
        ("fish", "fish"),
        ("ps1", "powershell"),
        ("svg", "svg"),
        ("xml", "markup"),
        ("graphql", "graphql"),
        ("gql", "graphql"),
        ("proto", "proto"),
        ("ex", "elixir"),
        ("exs", "elixir"),
        ("erl", "erlang"),
        ("hs", "haskell"),
        ("scala", "scala"),
        ("clj", "clojure"),
        ("cljs", "clojure"),
        ("nim", "nim"),
        ("zig", "zig"),
        ("vue", "vue"),
        ("svelte", "svelte"),
        ("elm", "elm"),
        ("rkt", "racket"),
        ("r", "rlang"),
        ("jl", "julia"),
        ("tex", "tex"),
        ("rst", "rst"),
        ("pdf", "pdf"),
        ("wasm", "wasm"),
        ("lock", "key"),
        ("log", "log"),
        ("env", "dotenv"),
        ("dockerfile", "docker"),
        ("dockerignore", "docker"),
        ("gitignore", "git"),
        ("gitattributes", "git"),
        ("gitmodules", "git"),
        ("editorconfig", "editorconfig"),
        ("eslintrc", "eslint"),
        ("prettierrc", "prettier"),
        ("png", "image"),
        ("jpg", "image"),
        ("jpeg", "image"),
        ("gif", "image"),
        ("ico", "image"),
        ("webp", "image"),
        ("bmp", "image"),
        ("mp3", "audio"),
        ("wav", "audio"),
        ("ogg", "audio"),
        ("flac", "audio"),
        ("mp4", "video"),
        ("mov", "video"),
        ("avi", "video"),
        ("mkv", "video"),
        ("zip", "zip"),
        ("tar", "zip"),
        ("gz", "zip"),
        ("rar", "zip"),
        ("exe", "exe"),
        ("bin", "binary"),
        ("dll", "binary"),
        ("csv", "excel"),
        ("xlsx", "excel"),
        ("xls", "excel"),
        ("doc", "word"),
        ("docx", "word"),
        ("pptx", "powerpoint"),
        ("nix", "nix"),
        ("sol", "solidity"),
        ("tf", "terraform"),
        ("prisma", "prisma"),
        ("gradle", "gradle"),
        ("groovy", "groovy"),
        ("f90", "fortran"),
        ("f95", "fortran"),
        ("d", "dlang"),
        ("cr", "crystal"),
        ("fs", "fsharp"),
        ("fsx", "fsharp"),
        ("ml", "ocaml"),
        ("mli", "ocaml"),
        ("hx", "haxe"),
        ("pas", "pascal"),
        ("pp", "pascal"),
        ("pug", "pug"),
        ("slim", "slim"),
        ("haml", "haml"),
        ("styl", "stylus"),
        ("postcss", "postcss"),
        ("coffee", "coffeescript"),
        ("litcoffee", "coffeescript"),
        ("asm", "assembly"),
        ("s", "assembly"),
        ("m", "objectivec"),
        ("mm", "objectivecpp"),
        ("ipynb", "jupyter"),
        ("rmd", "rstudio"),
        ("twig", "twig"),
        ("hbs", "handlebars"),
        ("mustache", "mustache"),
        ("ejs", "ejs"),
        ("njk", "nunjucks"),
        ("jinja", "jinja"),
        ("jinja2", "jinja"),
    ])
});

static FILE_NAME_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    HashMap::from([
        ("dockerfile", "docker"),
        ("docker-compose.yml", "docker"),
        ("docker-compose.yaml", "docker"),
        ("makefile", "gnu"),
        ("cmakelists.txt", "cmake"),
        (".gitignore", "git"),
        (".gitattributes", "git"),
        (".gitmodules", "git"),
        (".env", "dotenv"),
        (".env.local", "dotenv"),
        (".env.development", "dotenv"),
        (".env.production", "dotenv"),
        ("package.json", "npm"),
        ("package-lock.json", "npm"),
        ("cargo.toml", "rust"),
        ("cargo.lock", "rust"),
        ("tsconfig.json", "typescript"),
        ("jsconfig.json", "javascript"),
        (".eslintrc", "eslint"),
        (".eslintrc.js", "eslint"),
        (".eslintrc.json", "eslint"),
        (".prettierrc", "prettier"),
        (".prettierrc.js", "prettier"),
        (".prettierrc.json", "prettier"),
        ("jest.config.js", "jest"),
        ("jest.config.ts", "jest"),
        ("webpack.config.js", "webpack"),
        ("webpack.config.ts", "webpack"),
        ("rollup.config.js", "rollup"),
        ("rollup.config.ts", "rollup"),
        ("vite.config.js", "vitejs"),
        ("vite.config.ts", "vitejs"),
        ("next.config.js", "nextjs"),
        ("next.config.mjs", "nextjs"),
        ("tailwind.config.js", "tailwind"),
        ("tailwind.config.ts", "tailwind"),
        ("babel.config.js", "babel"),
        (".babelrc", "babel"),
        ("license", "certificate"),
        ("licence", "certificate"),
        ("readme.md", "markdown"),
        ("changelog.md", "markdown"),
        ("todo.md", "todo"),
        ("todo.txt", "todo"),
        ("todo", "todo"),
        (".editorconfig", "editorconfig"),
        ("yarn.lock", "yarn"),
        (".yarnrc", "yarn"),
        ("pnpm-lock.yaml", "pnpm"),
        (".npmrc", "npm"),
        ("nginx.conf", "nginx"),
        (".prettierignore", "prettier"),
        (".eslintignore", "eslint"),
        ("vercel.json", "vercel"),
        ("netlify.toml", "server"),
        (".dockerignore", "docker"),
    ])
});

static FOLDER_NAME_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    HashMap::from([
        ("src", "src"),
        ("source", "src"),
        ("lib", "src"),
        ("build", "build"),
        ("dist", "build"),
        ("out", "build"),
        ("output", "build"),
        ("node_modules", "node"),
        ("test", "tests"),
        ("tests", "tests"),
        ("__tests__", "tests"),
        ("spec", "tests"),
        (".git", "git"),
        ("styles", "styles"),
        ("css", "styles"),
        ("style", "styles"),
        ("images", "images"),
        ("img", "images"),
        ("assets", "images"),
        ("icons", "images"),
        ("views", "views"),
        ("pages", "views"),
        ("screens", "views"),
        ("db", "db"),
        ("database", "db"),
        ("migrations", "db"),
        ("i18n", "i18n"),
        ("locale", "i18n"),
        ("locales", "i18n"),
        ("lang", "i18n"),
        ("app", "app"),
        ("theme", "theme"),
        ("themes", "theme"),
        ("services", "services"),
        ("service", "services"),
        ("layout", "layout"),
        ("layouts", "layout"),
        (".vscode", "vsc"),
        ("bench", "bench"),
        ("benchmarks", "bench"),
        ("cypress", "cypress"),
        ("next", "next"),
        ("bower_components", "bower"),
        ("save", "save"),
        ("backup", "save"),
    ])
});

pub fn get_file_icon(filename: &str) -> IconAsset {
    let filename_lower = filename.to_lowercase();

    if let Some(&icon_name) = FILE_NAME_MAP.get(filename_lower.as_str()) {
        return resolve_icon("", icon_name);
    }

    // Try compound extension (e.g. "test.spec.ts" → "spec.ts")
    let parts: Vec<&str> = filename_lower.split('.').collect();
    for i in 1..parts.len() {
        let compound_ext = parts[i..].join(".");
        if let Some(&icon_name) = FILE_EXT_MAP.get(compound_ext.as_str()) {
            return resolve_icon("", icon_name);
        }
    }

    // Try simple extension
    if let Some(ext) = Path::new(filename).extension().and_then(|e| e.to_str()) {
        if let Some(&icon_name) = FILE_EXT_MAP.get(ext.to_lowercase().as_str()) {
            return resolve_icon("", icon_name);
        }
    }

    // Default file icon
    resolve_icon("", "file")
}

pub fn get_folder_icon(folder_name: &str, is_open: bool) -> IconAsset {
    let folder_lower = folder_name.to_lowercase();

    let icon_base_name = FOLDER_NAME_MAP
        .get(folder_lower.as_str())
        .copied()
        .unwrap_or("default");

    let name = if is_open {
        format!("{}-open", icon_base_name)
    } else {
        icon_base_name.to_string()
    };

    resolve_icon("folders", &name)
}
