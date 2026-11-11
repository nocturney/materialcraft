//! System fallback fonts for Arabic and Hebrew UI text.
//! Both scripts are requested when the language picker is visible, so their
//! native-script language names can be drawn before the user selects them.
//! No font binaries are bundled with MaterialCraft by this module.

use std::sync::Arc;

use egui::{FontData, FontFamily};

/// Existing system font candidates, ordered by preference; Hebrew and Arabic
/// have separate lists because an Arabic-only face cannot draw Hebrew.
#[cfg(not(target_arch = "wasm32"))]
fn system_files(hebrew: bool) -> Vec<(std::path::PathBuf, u32)> {
    use std::path::PathBuf;
    let mut v: Vec<(PathBuf, u32)> = Vec::new();
    if cfg!(windows) {
        let fonts = PathBuf::from(std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".into())).join("Fonts");
        let files: &[&str] = if hebrew { &["segoeui.ttf", "arial.ttf", "david.ttf", "tahoma.ttf"] } else { &["segoeui.ttf", "tahoma.ttf", "arial.ttf"] };
        v.extend(files.iter().map(|f| (fonts.join(f), 0)));
    } else if cfg!(target_os = "macos") {
        let files: &[&str] = if hebrew {
            &[
                "/System/Library/Fonts/SFHebrew.ttf",
                "/System/Library/Fonts/ArialHB.ttc",
                "/System/Library/Fonts/SFHebrewRounded.ttf",
                "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
            ]
        } else {
            &["/System/Library/Fonts/SFArabic.ttf", "/System/Library/Fonts/GeezaPro.ttc", "/System/Library/Fonts/Supplemental/Arial Unicode.ttf"]
        };
        v.extend(files.iter().map(|f| (PathBuf::from(f), 0)));
    } else {
        let files: &[&str] = if hebrew {
            &[
                "/usr/share/fonts/truetype/noto/NotoSansHebrew-Regular.ttf",
                "/usr/share/fonts/noto/NotoSansHebrew-Regular.ttf",
                "/usr/share/fonts/google-noto/NotoSansHebrew-Regular.ttf",
                "/usr/local/share/fonts/noto/NotoSansHebrew-Regular.ttf",
                "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
                "/usr/share/fonts/dejavu/DejaVuSans.ttf",
                "/usr/local/share/fonts/dejavu/DejaVuSans.ttf",
            ]
        } else {
            &[
                "/usr/share/fonts/truetype/noto/NotoSansArabic-Regular.ttf",
                "/usr/share/fonts/noto/NotoSansArabic-Regular.ttf",
                "/usr/share/fonts/google-noto/NotoSansArabic-Regular.ttf",
                "/usr/local/share/fonts/noto/NotoSansArabic-Regular.ttf",
                "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
                "/usr/share/fonts/dejavu/DejaVuSans.ttf",
                "/usr/local/share/fonts/dejavu/DejaVuSans.ttf",
            ]
        };
        v.extend(files.iter().map(|f| (PathBuf::from(f), 0)));
    }
    v
}

/// OS locale tags may include region/encoding suffixes, whereas Preferences
/// normally stores a two-letter language code.
fn matches_language(language: &str, code: &str) -> bool {
    language.split(['-', '_', '.']).next().is_some_and(|part| part.eq_ignore_ascii_case(code))
}

/// Loaded system faces persist as egui fallbacks when the user changes language.
#[derive(Default)]
pub struct UiFonts {
    attempted_arabic: bool,
    attempted_hebrew: bool,
    arabic: Option<Arc<FontData>>,
    hebrew: Option<Arc<FontData>>,
}

impl UiFonts {
    /// Install a font if its script is selected, or both while the picker is open.
    pub fn update(&mut self, ctx: &egui::Context, language: &str, preferences_open: bool) {
        let want_arabic = matches_language(language, "ar") || preferences_open;
        let want_hebrew = matches_language(language, "he") || preferences_open;
        let mut changed = false;
        if want_arabic && !self.attempted_arabic {
            self.attempted_arabic = true;
            self.arabic = Self::load_system(false);
            changed = true;
        }
        if want_hebrew && !self.attempted_hebrew {
            self.attempted_hebrew = true;
            self.hebrew = Self::load_system(true);
            changed = true;
        }
        if !changed {
            return;
        }

        let mut defs = crate::theme::font_definitions();
        let mut installed_any = false;
        for (name, data) in [("script-Arab", self.arabic.as_ref()), ("script-Hebr", self.hebrew.as_ref())] {
            if let Some(data) = data {
                let name = name.to_string();
                defs.font_data.insert(name.clone(), data.clone());
                defs.families.entry(FontFamily::Proportional).or_default().push(name.clone());
                defs.families.entry(FontFamily::Monospace).or_default().push(name);
                installed_any = true;
            }
        }
        if installed_any {
            ctx.set_fonts(defs);
        }
    }

    /// Whether an Arabic fallback was found and loaded.
    pub fn covers_arabic(&self) -> bool {
        self.arabic.is_some()
    }

    /// Whether a Hebrew fallback was found and loaded.
    pub fn covers_hebrew(&self) -> bool {
        self.hebrew.is_some()
    }

    fn load_system(hebrew: bool) -> Option<Arc<FontData>> {
        #[cfg(not(target_arch = "wasm32"))]
        for (path, index) in system_files(hebrew) {
            let Ok(meta) = std::fs::metadata(&path) else {
                continue;
            };
            if !meta.is_file() || meta.len() == 0 || meta.len() > (128 << 20) {
                continue;
            }
            if let Ok(bytes) = std::fs::read(&path) {
                let mut data = FontData::from_owned(bytes);
                data.index = index;
                let name = if hebrew { "Hebrew" } else { "Arabic" };
                log::info!("UI font fallback for {}: {}", name, path.display());
                return Some(Arc::new(data));
            }
        }
        let name = if hebrew { "Hebrew" } else { "Arabic" };
        log::warn!("no font found for {} UI text; system fonts may show boxes", name);
        None
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::{UiFonts, system_files};

    #[test]
    fn hebrew_native_font_fallback_loads_from_macos() {
        let candidates = system_files(true);
        assert!(candidates.iter().any(|(path, _)| path.ends_with("SFHebrew.ttf")));
        assert!(candidates.iter().any(|(path, _)| path.ends_with("ArialHB.ttc")));

        let mut fonts = UiFonts::default();
        fonts.update(&egui::Context::default(), "he-IL", false);
        assert!(fonts.covers_hebrew(), "missing Hebrew system UI fallback for he-IL");
    }
}

#[cfg(test)]
mod locale_tests {
    use super::matches_language;

    #[test]
    fn fallback_accepts_os_locale_variants_without_matching_other_languages() {
        for locale in ["he", "he-IL", "he_IL.UTF-8", "HE-il"] {
            assert!(matches_language(locale, "he"), "{locale}");
        }
        assert!(matches_language("ar-EG", "ar"));
        assert!(!matches_language("en-US", "he"));
        assert!(!matches_language("hi", "he"));
        assert!(!matches_language("ar-EG", "he"));
    }
}
