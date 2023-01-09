use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(Some(1))")]
pub enum Category {
    #[sea_orm(string_value = "B")]
    Big,
    #[sea_orm(string_value = "S")]
    Small,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum Color {
    #[sea_orm(num_value = 0)]
    Black,
    #[sea_orm(num_value = 1)]
    White,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "tea")]
pub enum Tea {
    #[sea_orm(string_value = "EverydayTea")]
    EverydayTea,
    #[sea_orm(string_value = "BreakfastTea")]
    BreakfastTea,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Copy)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "media_type")]
pub enum MediaType {
    #[sea_orm(string_value = "UNKNOWN")]
    Unknown,
    #[sea_orm(string_value = "BITMAP")]
    Bitmap,
    #[sea_orm(string_value = "DRAWING")]
    Drawing,
    #[sea_orm(string_value = "AUDIO")]
    Audio,
    #[sea_orm(string_value = "VIDEO")]
    Video,
    #[sea_orm(string_value = "MULTIMEDIA")]
    Multimedia,
    #[sea_orm(string_value = "OFFICE")]
    Office,
    #[sea_orm(string_value = "TEXT")]
    Text,
    #[sea_orm(string_value = "EXECUTABLE")]
    Executable,
    #[sea_orm(string_value = "ARCHIVE")]
    Archive,
    #[sea_orm(string_value = "3D")]
    _3D,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Copy)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "pop_os_names_typos")]
pub enum PopOSTypos {
    #[sea_orm(string_value = "Pop!_OS")]
    PopOSCorrect,
    #[sea_orm(string_value = "Pop\u{2757}_OS")]
    PopOSEmoji,
    #[sea_orm(string_value = "Pop!_操作系统")]
    PopOSChinese,
    #[sea_orm(string_value = "PopOS")]
    PopOSASCIIOnly,
    #[sea_orm(string_value = "Pop OS")]
    PopOSASCIIOnlyWithSpace,
    #[sea_orm(string_value = "Pop!OS")]
    PopOSNoUnderscore,
    #[sea_orm(string_value = "Pop_OS")]
    PopOSNoExclaimation,
    #[sea_orm(string_value = "!PopOS_")]
    PopOSAllOverThePlace,
    #[sea_orm(string_value = "Pop!_OS22.04LTS")]
    PopOSWithVersion,
    #[sea_orm(string_value = "22.04LTSPop!_OS")]
    PopOSWithVersionPrefix,
    #[sea_orm(string_value = "!_")]
    PopOSJustTheSymbols,
    #[sea_orm(string_value = "")]
    Nothing,
    // This WILL fail:
    // Both PopOS and PopOS will create identifier "PopOs"
    // #[sea_orm(string_value = "PopOs")]
    // PopOSLowerCase,
}
