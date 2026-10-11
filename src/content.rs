//! Site content: the profile and the project showcase.
//!
//! Content lives in `content/site.toml` and is compiled into the binary, so a
//! deploy is a single self-contained executable plus static assets. Invalid
//! content fails at startup (and in `cargo test`), never at request time.

use std::collections::HashSet;

use pulldown_cmark::{Options, Parser, html};
use serde::Deserialize;
use thiserror::Error;

const SITE_TOML: &str = include_str!("../content/site.toml");

#[derive(Debug, Error)]
pub enum ContentError {
    #[error("failed to parse site content: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("duplicate project slug {0:?}")]
    DuplicateSlug(String),
    #[error("invalid project slug {0:?}: use lowercase letters, digits and '-'")]
    InvalidSlug(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Active,
    Shipped,
    Experiment,
    Archived,
}

impl Status {
    pub fn label(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Shipped => "shipped",
            Self::Experiment => "experiment",
            Self::Archived => "archived",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Link {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Project {
    pub slug: String,
    pub title: String,
    pub summary: String,
    pub status: Status,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub featured: bool,
    pub repo: Option<String>,
    pub url: Option<String>,
    /// Long-form Markdown description, rendered to HTML at load time.
    #[serde(default, rename = "body")]
    body_markdown: String,
    #[serde(skip)]
    pub body_html: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Site {
    pub name: String,
    pub tagline: String,
    /// Markdown, rendered to HTML at load time.
    #[serde(rename = "about")]
    about_markdown: String,
    #[serde(skip)]
    pub about_html: String,
    #[serde(default)]
    pub links: Vec<Link>,
    #[serde(default, rename = "project")]
    pub projects: Vec<Project>,
}

impl Site {
    /// Load the content compiled into the binary.
    pub fn load() -> Result<Self, ContentError> {
        Self::parse(SITE_TOML)
    }

    /// Parse and validate site content from a TOML string.
    pub fn parse(source: &str) -> Result<Self, ContentError> {
        let mut site: Self = toml::from_str(source)?;

        let mut seen = HashSet::with_capacity(site.projects.len());
        for project in &mut site.projects {
            if !is_valid_slug(&project.slug) {
                return Err(ContentError::InvalidSlug(project.slug.clone()));
            }
            if !seen.insert(project.slug.clone()) {
                return Err(ContentError::DuplicateSlug(project.slug.clone()));
            }
            project.body_html = render_markdown(&project.body_markdown);
        }
        site.about_html = render_markdown(&site.about_markdown);

        Ok(site)
    }

    pub fn project(&self, slug: &str) -> Option<&Project> {
        self.projects.iter().find(|p| p.slug == slug)
    }

    pub fn featured(&self) -> impl Iterator<Item = &Project> {
        self.projects.iter().filter(|p| p.featured)
    }
}

fn is_valid_slug(slug: &str) -> bool {
    !slug.is_empty()
        && !slug.starts_with('-')
        && !slug.ends_with('-')
        && slug
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

// The Markdown is authored by the site owner and compiled into the binary, so
// it is trusted; raw HTML in it is passed through deliberately.
fn render_markdown(source: &str) -> String {
    let options = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH;
    let mut out = String::with_capacity(source.len() * 3 / 2);
    html::push_html(&mut out, Parser::new_ext(source, options));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL: &str = r#"
        name = "Test"
        tagline = "t"
        about = "Hello *world*"
    "#;

    fn with_projects(projects: &str) -> String {
        format!("{MINIMAL}\n{projects}")
    }

    #[test]
    fn load_should_accept_bundled_content() {
        assert!(Site::load().is_ok(), "{:?}", Site::load().err());
    }

    #[test]
    fn parse_should_render_about_markdown() {
        let site = Site::parse(MINIMAL).unwrap();
        assert_eq!(site.about_html.trim(), "<p>Hello <em>world</em></p>");
    }

    #[test]
    fn parse_should_reject_duplicate_slugs() {
        let source = with_projects(
            r#"
            [[project]]
            slug = "a"
            title = "A"
            summary = "s"
            status = "active"

            [[project]]
            slug = "a"
            title = "A again"
            summary = "s"
            status = "shipped"
            "#,
        );
        assert!(matches!(Site::parse(&source), Err(ContentError::DuplicateSlug(s)) if s == "a"));
    }

    #[test]
    fn parse_should_reject_slug_with_uppercase() {
        let source = with_projects(
            r#"
            [[project]]
            slug = "Bad"
            title = "B"
            summary = "s"
            status = "active"
            "#,
        );
        assert!(matches!(
            Site::parse(&source),
            Err(ContentError::InvalidSlug(_))
        ));
    }

    #[test]
    fn parse_should_reject_unknown_status() {
        let source = with_projects(
            r#"
            [[project]]
            slug = "a"
            title = "A"
            summary = "s"
            status = "someday"
            "#,
        );
        assert!(matches!(Site::parse(&source), Err(ContentError::Parse(_))));
    }
}
