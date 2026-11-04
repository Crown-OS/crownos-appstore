use crownos_apps_database::Category;
use crownui::kit::icons;

pub fn category_icon(category: Category) -> &'static str {
    match category {
        Category::Internet => icons::GLOBE,
        Category::Communication => icons::MESSAGE_CIRCLE,
        Category::Development => icons::CODE,
        Category::Graphics => icons::PALETTE,
        Category::Multimedia => icons::FILM,
        Category::Office => icons::BRIEFCASE,
        Category::Games => icons::GAMEPAD_2,
        Category::Education => icons::GRADUATION_CAP,
        Category::Utilities => icons::WRENCH,
        Category::System => icons::SHIELD,
    }
}
