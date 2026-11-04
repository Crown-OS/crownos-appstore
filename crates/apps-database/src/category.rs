use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Internet,
    Communication,
    Development,
    Graphics,
    Multimedia,
    Office,
    Games,
    Education,
    Utilities,
    System,
}

impl Category {
    pub const ALL: [Self; 10] = [
        Self::Internet,
        Self::Communication,
        Self::Development,
        Self::Graphics,
        Self::Multimedia,
        Self::Office,
        Self::Games,
        Self::Education,
        Self::Utilities,
        Self::System,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Internet => "Internet",
            Self::Communication => "Communication",
            Self::Development => "Development",
            Self::Graphics => "Graphics",
            Self::Multimedia => "Audio & Video",
            Self::Office => "Office",
            Self::Games => "Games",
            Self::Education => "Education",
            Self::Utilities => "Utilities",
            Self::System => "System",
        }
    }
}
