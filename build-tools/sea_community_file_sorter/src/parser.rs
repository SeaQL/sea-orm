use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    str::FromStr,
};

use regex::Regex;

/// Project struct, project is started with a dash. Project can have a repository and databases.
/// Project supports to be tow types:
/// - A project with a repository
/// - A project without a repository
/// ### Project with a repository
/// ```md
/// - [{project-name}]({project-link}) ([repository]({repository-link})) | {description} | DB: {databases separated by comma}
/// ```
/// ### Project without a repository
/// ```md
/// - [{project-name}]({project-link}) | {description} | DB: {databases separated by comma}
/// ```
/// Note: The `DB` part is optional
#[derive(Clone, PartialEq, Eq)]
pub struct Project {
    /// The name of the project
    pub name: String,
    /// The description of the project
    pub description: String,
    /// The link to the project
    pub link: String,
    /// The repository of the project
    pub repository_link: Option<String>,
    /// The list of databases that the project supports
    pub databases: Option<Vec<String>>,
}

/// Sub section struct, sub section is started with 4 hashes. Sub section can have projects.
#[derive(Clone, PartialEq, Eq)]
pub struct SubSection {
    /// The name of the sub section
    pub name: String,
    /// The list of projects
    pub projects: Vec<Project>,
}

/// Section struct, section is started with 3 hashes. Section can have sub sections.
#[derive(Clone, PartialEq, Eq)]
pub struct Section {
    /// The name of the section
    pub name: String,
    /// The list of sub sections
    pub sub_sections: Vec<SubSection>,
}

/// Community struct, this is the main struct that will be returned by the parser, this will contain the list of sections and learning resources.
#[derive(Clone, PartialEq, Eq)]
pub struct Community {
    /// The list of sections that are built with SeaORM
    /// The first element is the description of built with SeaORM, the second element is the list of sections
    pub built_with_sea_orm: (String, Vec<Section>),
    /// The list of learning resources, this will not be parsed
    pub learning_resources: String,
}

impl Community {
    /// Parse the community file, this will return a Community struct. This will panic if the file is not in the correct format.
    pub fn parse(file_path: &Path) -> Community {
        let file = File::open(file_path).expect("Failed to open file");
        let mut lines = BufReader::new(file).lines();
        if let Some(Ok(first_line)) = lines.next() {
            if first_line != "# Community" {
                panic!("First line is not `# Community`")
            }
        }
        let mut community = Community {
            built_with_sea_orm: (String::new(), Vec::new()),
            learning_resources: String::new(),
        };
        let mut is_learning_resources = false;
        // Is 2 because the count starts from 1, and the first line is already read
        let mut line_number = 2;

        for line in lines {
            let line = line.expect("Failed to read line");
            let line = line.trim();
            if line.is_empty() {
                line_number += 1;
                continue;
            }
            if is_learning_resources {
                community
                    .learning_resources
                    .push_str(&format!("{}\n", line));
            } else if let Some(main_section) = line.strip_prefix("## ") {
                if main_section == "Learning Resources" {
                    is_learning_resources = true;
                } else if main_section != "Built with SeaORM" {
                    panic!(
                        "line:{}: Unknown main section: {}",
                        line_number, main_section
                    )
                }
            } else if let Some(str_section) = line.strip_prefix("### ") {
                community.built_with_sea_orm.1.push(Section {
                    name: str_section.to_string(),
                    sub_sections: Vec::new(),
                });
            } else if let Some(str_sub_section) = line.strip_prefix("#### ") {
                let section = community
                    .built_with_sea_orm
                    .1
                    .last_mut()
                    .unwrap_or_else(|| {
                        panic!(
                            "line:{}: Found sub section without section: {}",
                            line_number, str_sub_section
                        )
                    });
                section.sub_sections.push(SubSection {
                    name: str_sub_section.to_string(),
                    projects: Vec::new(),
                });
            } else if let Some(str_project) = line.strip_prefix("- ") {
                let section = community
                    .built_with_sea_orm
                    .1
                    .last_mut()
                    .unwrap_or_else(|| {
                        panic!(
                            "line:{}: Found project without section: {}",
                            line_number, str_project
                        )
                    });
                let sub_section = section.sub_sections.last_mut().unwrap_or_else(|| {
                    panic!(
                        "line:{}: Found project without sub section: {}",
                        line_number, str_project
                    )
                });
                sub_section
                    .projects
                    .push(Project::from_str(str_project).unwrap_or_else(|err| {
                        panic!("line:{}: Failed to parse project: {}", line_number, err)
                    }));
            } else {
                // If the community is empty, then it's the description of built with SeaORM
                if community.built_with_sea_orm.1.is_empty() {
                    community
                        .built_with_sea_orm
                        .0
                        .push_str(&format!("{}\n", line));
                } else {
                    panic!("line:{}: Unknown line: {}", line_number, line);
                }
            }
            line_number += 1;
        }
        community
    }
}

impl FromStr for Project {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // A project parser from a string, parse with regex, with optional DB
        let s = s.trim();
        let re = Regex::new(
            r"^\[(?P<name>.+?)\]\((?P<link>.+?)\)( \((\[repository\]\((?P<repository_link>.+?)\))?\))? \| (?P<description>.+?)( \| DB: (?P<databases>.+?))?$",
        ).unwrap();
        let captures = re
            .captures(s)
            .ok_or_else(|| {
                format!("Failed to parse project: `{}`, please check the format. The format should be: `- [{{project-name}}]({{project-link}}) ([repository]({{repository-link}})) | {{description}} | DB: {{databases separated by comma}}`", s)
            })?;
        let name = captures
            .name("name")
            .ok_or("Failed to get project name")?
            .as_str()
            .to_string();
        let link = captures
            .name("link")
            .ok_or("Failed to get project link")?
            .as_str()
            .to_string();
        let repository_link = captures
            .name("repository_link")
            .map(|repository_link| repository_link.as_str().to_string());
        let description = captures
            .name("description")
            .ok_or("Failed to get project description")?
            .as_str()
            .to_string();
        let databases = captures.name("databases").map(|databases| {
            databases
                .as_str()
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        });
        Ok(Project {
            name,
            description,
            link,
            repository_link,
            databases,
        })
    }
}

impl ToString for Project {
    fn to_string(&self) -> String {
        let mut s = format!("- [{}]({})", self.name, self.link);
        if let Some(repository_link) = &self.repository_link {
            s.push_str(&format!(" ([repository]({}))", repository_link));
        }
        s.push_str(&format!(" | {}", self.description));
        if let Some(databases) = &self.databases {
            s.push_str(&format!(" | DB: {}", databases.join(", ")));
        }
        s
    }
}

impl ToString for SubSection {
    fn to_string(&self) -> String {
        format!(
            "#### {}\n{}",
            self.name,
            self.projects
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

impl ToString for Section {
    fn to_string(&self) -> String {
        format!(
            "### {}\n{}",
            self.name,
            self.sub_sections
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

impl ToString for Community {
    fn to_string(&self) -> String {
        format!(
            "# Community\n## Built with SeaORM\n{}\n{}\n## Learning Resources\n{}",
            self.built_with_sea_orm.0,
            self.built_with_sea_orm
                .1
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n"),
            self.learning_resources
        )
    }
}
