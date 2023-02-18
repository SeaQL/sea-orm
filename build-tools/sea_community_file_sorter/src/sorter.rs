use crate::parser::{Community, Section, SubSection};

/// Convert a diff to a string, for printing
fn diff_to_string(lines: Vec<diff::Result<&&str>>) -> String {
    lines
        .iter()
        .map(|line| match line {
            diff::Result::Left(l) => format!("-{}", l),
            diff::Result::Both(l, _) => l.to_string(),
            diff::Result::Right(r) => format!("+{}", r),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Trait for sorting and checking if it's sorted or not, alphabetically
pub trait AlphapeticalSorter {
    #[must_use = "This will not modify, will return a new sorted one"]
    fn sort(&self) -> Self;
    fn check_sorted(&self) -> Result<(), String>;
}

impl AlphapeticalSorter for SubSection {
    /// Sort the projects of this sub-section
    fn sort(&self) -> Self {
        let mut projects = self.projects.clone();
        projects.sort_by(|a, b| a.name.cmp(&b.name));
        SubSection {
            name: self.name.clone(),
            projects,
        }
    }

    /// Check if the projects of this sub-section are sorted or not
    fn check_sorted(&self) -> Result<(), String> {
        let sub_section = self.sort();
        if self == &sub_section {
            Ok(())
        } else {
            let left = &self
                .projects
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>();
            let right = &sub_section
                .projects
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>();
            let diff = diff::slice(left, right);

            Err(format!(
                "Projects of `{}` sub-section are not sorted.\n{}",
                self.name,
                diff_to_string(diff)
            ))
        }
    }
}

impl AlphapeticalSorter for Section {
    /// Sort the sub-sections of this section
    fn sort(&self) -> Self {
        let mut sub_sections: Vec<_> = self.sub_sections.iter().map(|s| s.sort()).collect();
        sub_sections.sort_by(|a, b| a.name.cmp(&b.name));
        Section {
            name: self.name.clone(),
            sub_sections,
        }
    }

    /// Check if the sub-sections of this section are sorted or not
    fn check_sorted(&self) -> Result<(), String> {
        let mut sub_sections = self.sub_sections.clone();
        sub_sections.sort_by(|a, b| a.name.cmp(&b.name));
        if self.sub_sections == sub_sections {
            // If the sub-sections are sorted, check if the projects are sorted
            for sub_section in &self.sub_sections {
                sub_section.check_sorted()?;
            }
            Ok(())
        } else {
            Err(format!(
                "Sub-sections of `{}` section are not sorted.\n{}",
                self.name,
                diff_to_string(diff::slice(
                    &self
                        .sub_sections
                        .iter()
                        .map(|p| p.name.as_str())
                        .collect::<Vec<_>>(),
                    &sub_sections
                        .iter()
                        .map(|p| p.name.as_str())
                        .collect::<Vec<_>>()
                ))
            ))
        }
    }
}

impl AlphapeticalSorter for Community {
    /// Sort the sections of this community
    fn sort(&self) -> Self {
        let mut sections: Vec<_> = self.built_with_sea_orm.1.iter().map(|s| s.sort()).collect();
        sections.sort_by(|a, b| a.name.cmp(&b.name));
        Community {
            built_with_sea_orm: (self.built_with_sea_orm.0.clone(), sections),
            learning_resources: self.learning_resources.clone(),
        }
    }

    /// Check if the sections of this community are sorted or not
    fn check_sorted(&self) -> Result<(), String> {
        let mut sections = self.built_with_sea_orm.1.clone();
        sections.sort_by(|a, b| a.name.cmp(&b.name));
        if self.built_with_sea_orm.1 == sections {
            // if the sections are sorted, then check if the sub-sections are sorted
            for section in &self.built_with_sea_orm.1 {
                section.check_sorted()?;
            }
            Ok(())
        } else {
            Err(format!(
                "Sections are not sorted.\n{}",
                diff_to_string(diff::slice(
                    &self
                        .built_with_sea_orm
                        .1
                        .iter()
                        .map(|p| p.name.as_str())
                        .collect::<Vec<_>>(),
                    &sections.iter().map(|p| p.name.as_str()).collect::<Vec<_>>()
                ))
            ))
        }
    }
}
