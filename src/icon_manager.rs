use eframe::egui;
use std::collections::HashMap;
use usvg::FitTo;

pub struct IconManager {
    icons: HashMap<String, egui::TextureHandle>,
    missing_icons: HashMap<String, bool>,
}

impl IconManager {
    pub fn new() -> Self {
        Self {
            icons: HashMap::new(),
            missing_icons: HashMap::new(),
        }
    }

    pub fn load_icon(&mut self, ctx: &egui::Context, key: &str, icon_path: &str) {
        if self.icons.contains_key(key) || self.missing_icons.contains_key(key) {
            return;
        }

        // Try to load from embedded assets first
        let icon_data = match self.load_embedded_asset(icon_path) {
            Some(data) => data,
            None => {
                // Fallback to filesystem
                match std::fs::read(icon_path) {
                    Ok(data) => data,
                    Err(_) => {
                        self.missing_icons.insert(key.to_string(), true);
                        return;
                    }
                }
            }
        };

        let color_image = if icon_path.ends_with(".png") {
            match image::load_from_memory(&icon_data) {
                Ok(img) => {
                    let rgba = img.to_rgba8();
                    let size = [rgba.width() as usize, rgba.height() as usize];
                    egui::ColorImage::from_rgba_unmultiplied(size, &rgba)
                }
                Err(_) => {
                    self.missing_icons.insert(key.to_string(), true);
                    return;
                }
            }
        } else {
            let opt = usvg::Options::default();
            let tree = match usvg::Tree::from_data(&icon_data, &opt) {
                Ok(t) => t,
                Err(_) => {
                    self.missing_icons.insert(key.to_string(), true);
                    return;
                }
            };

            let mut pixmap = match resvg::tiny_skia::Pixmap::new(16, 16) {
                Some(p) => p,
                None => return,
            };

            resvg::render(
                &tree,
                FitTo::Size(16, 16),
                resvg::tiny_skia::Transform::default(),
                pixmap.as_mut(),
            );

            egui::ColorImage::from_rgba_unmultiplied([16, 16], pixmap.data())
        };

        let texture = ctx.load_texture(key, color_image, egui::TextureOptions::LINEAR);

        self.icons.insert(key.to_string(), texture);
    }

    fn load_embedded_asset(&self, path: &str) -> Option<Vec<u8>> {
        let asset_path = path.strip_prefix("assets/").unwrap_or(path);

        match asset_path {
            "icons/default.png" => Some(include_bytes!("../assets/icons/default.png").to_vec()),

            "icons/rust.svg" => Some(include_bytes!("../assets/icons/rust.svg").to_vec()),
            "icons/javascript.svg" => {
                Some(include_bytes!("../assets/icons/javascript.svg").to_vec())
            }
            "icons/typescript.svg" => {
                Some(include_bytes!("../assets/icons/typescript.svg").to_vec())
            }
            "icons/python.svg" => Some(include_bytes!("../assets/icons/python.svg").to_vec()),
            "icons/markdown.svg" => Some(include_bytes!("../assets/icons/markdown.svg").to_vec()),
            "icons/json.svg" => Some(include_bytes!("../assets/icons/json.svg").to_vec()),
            "icons/toml.svg" => Some(include_bytes!("../assets/icons/toml.png").to_vec()),
            "icons/html.svg" => Some(include_bytes!("../assets/icons/html.svg").to_vec()),
            "icons/css.svg" => Some(include_bytes!("../assets/icons/css.svg").to_vec()),
            "icons/sass.svg" => Some(include_bytes!("../assets/icons/sass.svg").to_vec()),
            "icons/less.svg" => Some(include_bytes!("../assets/icons/less.svg").to_vec()),
            "icons/java.svg" => Some(include_bytes!("../assets/icons/java.svg").to_vec()),
            "icons/cpp.svg" => Some(include_bytes!("../assets/icons/cpp.svg").to_vec()),
            "icons/c.svg" => Some(include_bytes!("../assets/icons/c.png").to_vec()),
            "icons/cpp-h.svg" => Some(include_bytes!("../assets/icons/cpp-h.png").to_vec()),
            "icons/csharp.svg" => Some(include_bytes!("../assets/icons/csharp.svg").to_vec()),
            "icons/go.svg" => Some(include_bytes!("../assets/icons/go.svg").to_vec()),
            "icons/ruby.svg" => Some(include_bytes!("../assets/icons/ruby.svg").to_vec()),
            "icons/php.svg" => Some(include_bytes!("../assets/icons/php.svg").to_vec()),
            "icons/sql.svg" | "icons/database.svg" => {
                Some(include_bytes!("../assets/icons/database.svg").to_vec())
            }
            "icons/shell.svg" => Some(include_bytes!("../assets/icons/shell.png").to_vec()),
            "icons/yaml.svg" => Some(include_bytes!("../assets/icons/yaml.svg").to_vec()),
            "icons/xml.svg" | "icons/markup.svg" => {
                Some(include_bytes!("../assets/icons/markup.png").to_vec())
            }
            "icons/svg.svg" => Some(include_bytes!("../assets/icons/svg.svg").to_vec()),
            "icons/vue.svg" => Some(include_bytes!("../assets/icons/vue.svg").to_vec()),
            "icons/react.svg" => Some(include_bytes!("../assets/icons/react.svg").to_vec()),
            "icons/kotlin.svg" => Some(include_bytes!("../assets/icons/kotlin.svg").to_vec()),
            "icons/swift.svg" => Some(include_bytes!("../assets/icons/swift.svg").to_vec()),
            "icons/dart.svg" => Some(include_bytes!("../assets/icons/dart.svg").to_vec()),
            "icons/rlang.svg" => Some(include_bytes!("../assets/icons/rlang.png").to_vec()),
            "icons/lua.svg" => Some(include_bytes!("../assets/icons/lua.svg").to_vec()),
            "icons/elixir.svg" => Some(include_bytes!("../assets/icons/elixir.svg").to_vec()),
            "icons/elm.svg" => Some(include_bytes!("../assets/icons/elm.svg").to_vec()),
            "icons/perl.svg" => Some(include_bytes!("../assets/icons/perl.png").to_vec()),
            "icons/clojure.svg" => Some(include_bytes!("../assets/icons/clojure.svg").to_vec()),
            "icons/scala.svg" => Some(include_bytes!("../assets/icons/scala.svg").to_vec()),
            "icons/haskell.svg" => Some(include_bytes!("../assets/icons/haskell.svg").to_vec()),
            "icons/coffeescript.svg" => {
                Some(include_bytes!("../assets/icons/coffeescript.svg").to_vec())
            }
            "icons/stylus.svg" => Some(include_bytes!("../assets/icons/stylus.svg").to_vec()),
            "icons/pug.svg" => Some(include_bytes!("../assets/icons/pug.png").to_vec()),
            "icons/ejs.svg" => Some(include_bytes!("../assets/icons/ejs.svg").to_vec()),
            "icons/handlebars.svg" => {
                Some(include_bytes!("../assets/icons/handlebars.svg").to_vec())
            }
            "icons/tex.svg" => Some(include_bytes!("../assets/icons/tex.svg").to_vec()),
            "icons/pdf.svg" => Some(include_bytes!("../assets/icons/pdf.svg").to_vec()),
            "icons/zip.svg" => Some(include_bytes!("../assets/icons/zip.svg").to_vec()),
            "icons/image.svg" => Some(include_bytes!("../assets/icons/image.png").to_vec()),
            "icons/video.svg" => Some(include_bytes!("../assets/icons/video.png").to_vec()),
            "icons/audio.svg" => Some(include_bytes!("../assets/icons/audio.png").to_vec()),
            "icons/log.svg" => Some(include_bytes!("../assets/icons/log.svg").to_vec()),
            "icons/dotenv.svg" => Some(include_bytes!("../assets/icons/dotenv.svg").to_vec()),
            "icons/notepad.svg" => Some(include_bytes!("../assets/icons/notepad.svg").to_vec()),
            "icons/wasm.svg" => Some(include_bytes!("../assets/icons/wasm.svg").to_vec()),
            "icons/prisma.svg" => Some(include_bytes!("../assets/icons/prisma.svg").to_vec()),
            "icons/graphql.svg" => Some(include_bytes!("../assets/icons/graphql.svg").to_vec()),
            "icons/svelte.svg" => Some(include_bytes!("../assets/icons/svelte.svg").to_vec()),
            "icons/vim.svg" => Some(include_bytes!("../assets/icons/vim.svg").to_vec()),
            "icons/key.svg" => Some(include_bytes!("../assets/icons/key.png").to_vec()),
            "icons/git.svg" => Some(include_bytes!("../assets/icons/git.svg").to_vec()),
            "icons/docker.svg" => Some(include_bytes!("../assets/icons/docker.svg").to_vec()),
            "icons/nginx.svg" => Some(include_bytes!("../assets/icons/nginx.svg").to_vec()),
            "icons/cmake.svg" => Some(include_bytes!("../assets/icons/cmake.svg").to_vec()),
            "icons/settings.svg" => Some(include_bytes!("../assets/icons/settings.svg").to_vec()),
            "icons/gradle.svg" => Some(include_bytes!("../assets/icons/gradle.svg").to_vec()),
            "icons/powershell.svg" => {
                Some(include_bytes!("../assets/icons/powershell.png").to_vec())
            }
            "icons/fish.svg" => Some(include_bytes!("../assets/icons/fish.png").to_vec()),
            "icons/zig.svg" => Some(include_bytes!("../assets/icons/zig.svg").to_vec()),
            "icons/nim.svg" => Some(include_bytes!("../assets/icons/nim.svg").to_vec()),
            "icons/verilog.svg" => Some(include_bytes!("../assets/icons/verilog.svg").to_vec()),
            "icons/verilog-sys.svg" => {
                Some(include_bytes!("../assets/icons/verilog-sys.svg").to_vec())
            }
            "icons/erlang.svg" => Some(include_bytes!("../assets/icons/erlang.png").to_vec()),
            "icons/fsharp.svg" => Some(include_bytes!("../assets/icons/fsharp.svg").to_vec()),
            "icons/dlang.svg" => Some(include_bytes!("../assets/icons/dlang.png").to_vec()),
            "icons/pascal.svg" => Some(include_bytes!("../assets/icons/pascal.png").to_vec()),
            "icons/xaml.svg" => Some(include_bytes!("../assets/icons/xaml.png").to_vec()),
            "icons/visualstudio.svg" => {
                Some(include_bytes!("../assets/icons/visualstudio.svg").to_vec())
            }
            "icons/proto.svg" => Some(include_bytes!("../assets/icons/proto.svg").to_vec()),
            "icons/terraform.svg" => Some(include_bytes!("../assets/icons/terraform.svg").to_vec()),
            "icons/blade.svg" => Some(include_bytes!("../assets/icons/blade.svg").to_vec()),
            "icons/twig.svg" => Some(include_bytes!("../assets/icons/twig.png").to_vec()),
            "icons/liquid.svg" => Some(include_bytes!("../assets/icons/liquid.svg").to_vec()),
            "icons/nunjucks.svg" => Some(include_bytes!("../assets/icons/nunjucks.svg").to_vec()),
            "icons/jinja.svg" => Some(include_bytes!("../assets/icons/jinja.svg").to_vec()),
            "icons/slim.svg" => Some(include_bytes!("../assets/icons/slim.svg").to_vec()),
            "icons/haml.svg" => Some(include_bytes!("../assets/icons/haml.png").to_vec()),
            "icons/exe.svg" => Some(include_bytes!("../assets/icons/exe.png").to_vec()),
            "icons/binary.svg" => Some(include_bytes!("../assets/icons/binary.png").to_vec()),
            "icons/npm.svg" => Some(include_bytes!("../assets/icons/npm.svg").to_vec()),
            "icons/webpack.svg" => Some(include_bytes!("../assets/icons/webpack.svg").to_vec()),
            "icons/gulp.svg" => Some(include_bytes!("../assets/icons/gulp.svg").to_vec()),
            "icons/grunt.svg" => Some(include_bytes!("../assets/icons/grunt.svg").to_vec()),
            "icons/eslint.svg" => Some(include_bytes!("../assets/icons/eslint.svg").to_vec()),
            "icons/prettier.svg" => Some(include_bytes!("../assets/icons/prettier.svg").to_vec()),
            "icons/yarn.svg" => Some(include_bytes!("../assets/icons/yarn.svg").to_vec()),
            "icons/certificate.svg" => {
                Some(include_bytes!("../assets/icons/certificate.png").to_vec())
            }
            "icons/editorconfig.svg" => {
                Some(include_bytes!("../assets/icons/editorconfig.png").to_vec())
            }
            "icons/contributing.svg" => {
                Some(include_bytes!("../assets/icons/contributing.png").to_vec())
            }
            "icons/jest.svg" => Some(include_bytes!("../assets/icons/jest.svg").to_vec()),
            "icons/cypress.svg" => Some(include_bytes!("../assets/icons/cypress.svg").to_vec()),
            "icons/babel.svg" => Some(include_bytes!("../assets/icons/babel.svg").to_vec()),
            "icons/rollup.svg" => Some(include_bytes!("../assets/icons/rollup.svg").to_vec()),
            "icons/vitejs.svg" => Some(include_bytes!("../assets/icons/vitejs.svg").to_vec()),
            "icons/turborepo.svg" => Some(include_bytes!("../assets/icons/turborepo.svg").to_vec()),
            "icons/pnpm.svg" => Some(include_bytes!("../assets/icons/pnpm.svg").to_vec()),
            "icons/firebase.svg" => Some(include_bytes!("../assets/icons/firebase.svg").to_vec()),
            "icons/nextjs.svg" => Some(include_bytes!("../assets/icons/nextjs.svg").to_vec()),
            "icons/nuget.svg" => Some(include_bytes!("../assets/icons/nuget.svg").to_vec()),
            "icons/tailwind.svg" => Some(include_bytes!("../assets/icons/tailwind.svg").to_vec()),
            "icons/json5.svg" => Some(include_bytes!("../assets/icons/json5.svg").to_vec()),
            "icons/mdx.svg" => Some(include_bytes!("../assets/icons/mdx.svg").to_vec()),

            "icons/actionscript.png" => {
                Some(include_bytes!("../assets/icons/actionscript.png").to_vec())
            }
            "icons/axure.png" => Some(include_bytes!("../assets/icons/axure.png").to_vec()),
            "icons/buckle.png" => Some(include_bytes!("../assets/icons/buckle.png").to_vec()),
            "icons/c-h.png" => Some(include_bytes!("../assets/icons/c-h.png").to_vec()),
            "icons/cake.png" => Some(include_bytes!("../assets/icons/cake.png").to_vec()),
            "icons/cfc.png" => Some(include_bytes!("../assets/icons/cfc.png").to_vec()),
            "icons/cfm.png" => Some(include_bytes!("../assets/icons/cfm.png").to_vec()),
            "icons/coldfusion.png" => {
                Some(include_bytes!("../assets/icons/coldfusion.png").to_vec())
            }
            "icons/cursor.png" => Some(include_bytes!("../assets/icons/cursor.png").to_vec()),
            "icons/d.png" => Some(include_bytes!("../assets/icons/d.png").to_vec()),
            "icons/denizen.png" => Some(include_bytes!("../assets/icons/denizen.png").to_vec()),
            "icons/docz.png" => Some(include_bytes!("../assets/icons/docz.png").to_vec()),
            "icons/dust.png" => Some(include_bytes!("../assets/icons/dust.png").to_vec()),
            "icons/file.png" => Some(include_bytes!("../assets/icons/file.png").to_vec()),
            "icons/foxpro.png" => Some(include_bytes!("../assets/icons/foxpro.png").to_vec()),
            "icons/gherkin.png" => Some(include_bytes!("../assets/icons/gherkin.png").to_vec()),
            "icons/graphviz.png" => Some(include_bytes!("../assets/icons/graphviz.png").to_vec()),
            "icons/icon.png" => Some(include_bytes!("../assets/icons/icon.png").to_vec()),
            "icons/markojs.png" => Some(include_bytes!("../assets/icons/markojs.png").to_vec()),
            "icons/mustache.png" => Some(include_bytes!("../assets/icons/mustache.png").to_vec()),
            "icons/nextflow.png" => Some(include_bytes!("../assets/icons/nextflow.png").to_vec()),
            "icons/oso.png" => Some(include_bytes!("../assets/icons/oso.png").to_vec()),
            "icons/paket.png" => Some(include_bytes!("../assets/icons/paket.png").to_vec()),
            "icons/pascal-project.png" => {
                Some(include_bytes!("../assets/icons/pascal-project.svg").to_vec())
            }
            "icons/png.png" => Some(include_bytes!("../assets/icons/image.png").to_vec()),
            "icons/razor.png" => Some(include_bytes!("../assets/icons/razor.png").to_vec()),
            "icons/rst.png" => Some(include_bytes!("../assets/icons/rst.png").to_vec()),
            "icons/server.png" => Some(include_bytes!("../assets/icons/server.png").to_vec()),
            "icons/serverless.png" => {
                Some(include_bytes!("../assets/icons/serverless.png").to_vec())
            }
            "icons/smarty.png" => Some(include_bytes!("../assets/icons/smarty.png").to_vec()),
            "icons/stylelint.png" => Some(include_bytes!("../assets/icons/stylelint.png").to_vec()),
            "icons/tcl.png" => Some(include_bytes!("../assets/icons/tcl.png").to_vec()),
            "icons/textile.png" => Some(include_bytes!("../assets/icons/textile.png").to_vec()),
            "icons/url.png" => Some(include_bytes!("../assets/icons/url.png").to_vec()),

            "icons/folders/default.svg" => {
                Some(include_bytes!("../assets/icons/folders/default.svg").to_vec())
            }
            "icons/folders/default-open.svg" => {
                Some(include_bytes!("../assets/icons/folders/default-open.svg").to_vec())
            }
            "icons/folders/src.svg" => {
                Some(include_bytes!("../assets/icons/folders/src.svg").to_vec())
            }
            "icons/folders/src-open.svg" => {
                Some(include_bytes!("../assets/icons/folders/src-open.svg").to_vec())
            }
            "icons/folders/tests.svg" => {
                Some(include_bytes!("../assets/icons/folders/tests.svg").to_vec())
            }
            "icons/folders/tests-open.svg" => {
                Some(include_bytes!("../assets/icons/folders/tests-open.svg").to_vec())
            }
            "icons/folders/node.svg" => {
                Some(include_bytes!("../assets/icons/folders/node.svg").to_vec())
            }
            "icons/folders/node-open.svg" => {
                Some(include_bytes!("../assets/icons/folders/node-open.svg").to_vec())
            }
            "icons/folders/git.svg" => {
                Some(include_bytes!("../assets/icons/folders/git.svg").to_vec())
            }
            "icons/folders/git-open.svg" => {
                Some(include_bytes!("../assets/icons/folders/git-open.svg").to_vec())
            }
            "icons/folders/images.svg" => {
                Some(include_bytes!("../assets/icons/folders/images.svg").to_vec())
            }
            "icons/folders/images-open.svg" => {
                Some(include_bytes!("../assets/icons/folders/images-open.svg").to_vec())
            }
            "icons/folders/build.svg" => {
                Some(include_bytes!("../assets/icons/folders/build.svg").to_vec())
            }
            "icons/folders/build-open.svg" => {
                Some(include_bytes!("../assets/icons/folders/build-open.svg").to_vec())
            }
            "icons/folders/bower.svg" => {
                Some(include_bytes!("../assets/icons/folders/bower.svg").to_vec())
            }
            "icons/folders/bower-open.svg" => {
                Some(include_bytes!("../assets/icons/folders/bower-open.svg").to_vec())
            }
            "icons/folders/app.svg" => {
                Some(include_bytes!("../assets/icons/folders/app.svg").to_vec())
            }
            "icons/folders/app-open.svg" => {
                Some(include_bytes!("../assets/icons/folders/app-open.svg").to_vec())
            }
            "icons/folders/views.svg" => {
                Some(include_bytes!("../assets/icons/folders/views.svg").to_vec())
            }
            "icons/folders/views-open.svg" => {
                Some(include_bytes!("../assets/icons/folders/views-open.svg").to_vec())
            }
            "icons/folders/services.svg" => {
                Some(include_bytes!("../assets/icons/folders/services.svg").to_vec())
            }
            "icons/folders/services-open.svg" => {
                Some(include_bytes!("../assets/icons/folders/services-open.svg").to_vec())
            }
            "icons/folders/styles.svg" => {
                Some(include_bytes!("../assets/icons/folders/styles.svg").to_vec())
            }
            "icons/folders/styles-open.svg" => {
                Some(include_bytes!("../assets/icons/folders/styles-open.svg").to_vec())
            }
            "icons/folders/i18n.svg" => {
                Some(include_bytes!("../assets/icons/folders/i18n.svg").to_vec())
            }
            "icons/folders/i18n-open.svg" => {
                Some(include_bytes!("../assets/icons/folders/i18n-open.svg").to_vec())
            }
            "icons/folders/db.svg" => {
                Some(include_bytes!("../assets/icons/folders/db.svg").to_vec())
            }
            "icons/folders/db-open.svg" => {
                Some(include_bytes!("../assets/icons/folders/db-open.svg").to_vec())
            }
            "icons/folders/cypress.svg" => {
                Some(include_bytes!("../assets/icons/folders/cypress.svg").to_vec())
            }
            "icons/folders/cypress-open.svg" => {
                Some(include_bytes!("../assets/icons/folders/cypress-open.svg").to_vec())
            }
            "icons/folders/jest.svg" => {
                Some(include_bytes!("../assets/icons/folders/jest.svg").to_vec())
            }
            "icons/folders/jest-open.svg" => {
                Some(include_bytes!("../assets/icons/folders/jest-open.svg").to_vec())
            }
            "icons/folders/next.svg" => {
                Some(include_bytes!("../assets/icons/folders/next.svg").to_vec())
            }
            "icons/folders/next-open.svg" => {
                Some(include_bytes!("../assets/icons/folders/next-open.svg").to_vec())
            }
            "icons/folders/bench.svg" => {
                Some(include_bytes!("../assets/icons/folders/bench.svg").to_vec())
            }
            "icons/folders/bench-open.svg" => {
                Some(include_bytes!("../assets/icons/folders/bench-open.svg").to_vec())
            }
            "icons/folders/layout.svg" => {
                Some(include_bytes!("../assets/icons/folders/layout.svg").to_vec())
            }
            "icons/folders/layout-open.svg" => {
                Some(include_bytes!("../assets/icons/folders/layout-open.svg").to_vec())
            }
            "icons/folders/theme.svg" => {
                Some(include_bytes!("../assets/icons/folders/theme.svg").to_vec())
            }
            "icons/folders/theme-open.svg" => {
                Some(include_bytes!("../assets/icons/folders/theme-open.svg").to_vec())
            }
            "icons/folders/vsc.svg" => {
                Some(include_bytes!("../assets/icons/folders/vsc.svg").to_vec())
            }
            "icons/folders/vsc-open.svg" => {
                Some(include_bytes!("../assets/icons/folders/vsc-open.svg").to_vec())
            }

            _ => None,
        }
    }

    fn ensure_default_icon(&mut self, ctx: &egui::Context) {
        if !self.icons.contains_key("default") {
            let default_path = "assets/icons/default.png";
            self.load_icon(ctx, "default", default_path);

            if !self.icons.contains_key("default") {
                let mut pixmap = tiny_skia::Pixmap::new(16, 16).unwrap();
                pixmap.fill(tiny_skia::Color::from_rgba8(128, 128, 128, 255));

                let color_image = egui::ColorImage::from_rgba_unmultiplied([16, 16], pixmap.data());
                let texture =
                    ctx.load_texture("default", color_image, egui::TextureOptions::LINEAR);
                self.icons.insert("default".to_string(), texture);
            }
        }
    }

    pub fn get_file_icon(&mut self, ctx: &egui::Context, filename: &str) -> &egui::TextureHandle {
        let icon_path = crate::icon_theme::get_file_icon_path(filename);
        let key = format!("file:{}", icon_path);

        if !self.icons.contains_key(&key) && !self.missing_icons.contains_key(&key) {
            self.load_icon(ctx, &key, &icon_path);
        }

        if self.icons.contains_key(&key) {
            return self.icons.get(&key).unwrap();
        }

        self.ensure_default_icon(ctx);
        self.icons.get("default").unwrap()
    }

    pub fn get_folder_icon(
        &mut self,
        ctx: &egui::Context,
        folder_name: &str,
        is_open: bool,
    ) -> &egui::TextureHandle {
        let icon_path = crate::icon_theme::get_folder_icon_path(folder_name, is_open);
        let key = format!("folder:{}", icon_path);

        if !self.icons.contains_key(&key) && !self.missing_icons.contains_key(&key) {
            self.load_icon(ctx, &key, &icon_path);
        }

        if self.icons.contains_key(&key) {
            return self.icons.get(&key).unwrap();
        }

        let fallback_path = format!(
            "assets/icons/folders/{}.svg",
            if is_open { "default-open" } else { "default" }
        );
        let fallback_key = format!("folder:{}", fallback_path);

        if !self.icons.contains_key(&fallback_key)
            && !self.missing_icons.contains_key(&fallback_key)
        {
            self.load_icon(ctx, &fallback_key, &fallback_path);
        }

        if self.icons.contains_key(&fallback_key) {
            return self.icons.get(&fallback_key).unwrap();
        }

        self.ensure_default_icon(ctx);
        self.icons.get("default").unwrap()
    }
}
