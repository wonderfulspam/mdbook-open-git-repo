use std::path::{Path, PathBuf};

use mdbook::book::{Book, BookItem, Chapter};
use mdbook::errors::Result;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};

#[derive(Debug)]
enum SourceControlHost {
    GitHub,
    GitLab,
}

static DEFAULT_LINK_TEXT: &str = "Edit this file on GitHub.";
static DEFAULT_EDIT_TEXT: &str = "Found a bug? ";

pub struct OpenOn;

impl Preprocessor for OpenOn {
    fn name(&self) -> &str {
        "open-git-repo"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        let book_root = &ctx.root;
        let src_root = book_root.join(&ctx.config.book.src);
        let git_root = find_git(book_root).unwrap();
        log::debug!("Book root: {}", book_root.display());
        log::debug!("Src root: {}", src_root.display());
        log::debug!("Git root: {}", git_root.display());

        let repository_url = match ctx.config.get("output.html.git-repository-url") {
            None => return Ok(book),
            Some(url) => url,
        };
        let repository_url = match repository_url {
            toml::Value::String(s) => s,
            _ => return Ok(book),
        };
        log::debug!("Repository URL: {}", repository_url);

        let preprocessor_config = ctx.config.get_preprocessor(self.name());

        let source_control_host: SourceControlHost = match preprocessor_config {
            None => match repo_url_to_host(repository_url) {
                Some(host) => host,
                None => {
                    eprintln!("Failed to determine source control host from URL. Please specify the host in your configuration");
                    return Ok(book);
                }
            },
            Some(preprocessor_config) => {
                if let Some(toml::Value::String(host_string)) =
                    preprocessor_config.get("source-control-host")
                {
                    match host_string.as_ref() {
                        "github" => SourceControlHost::GitHub,
                        "gitlab" => SourceControlHost::GitLab,
                        _ => {
                            eprintln!("Invalid source control host. Please consult configuration guide for valid values");
                            return Ok(book);
                        }
                    }
                } else {
                    match repo_url_to_host(repository_url) {
                        Some(host) => host,
                        None => {
                            eprintln!("Failed to determine source control host from URL. Please specify the host in your configuration");
                            return Ok(book);
                        }
                    }
                }
            }
        };
        log::debug!("Source Control Host: {:?}", source_control_host);

        let link_text = match preprocessor_config {
            None => DEFAULT_LINK_TEXT,
            Some(preprocessor_config) => {
                if let Some(toml::Value::String(link_text)) = preprocessor_config.get("link-text") {
                    link_text
                } else {
                    DEFAULT_LINK_TEXT
                }
            }
        };
        log::debug!("Link Text: {}", link_text);

        let edit_text = match preprocessor_config {
            None => DEFAULT_EDIT_TEXT,
            Some(preprocessor_config) => {
                if let Some(toml::Value::String(edit_text)) = preprocessor_config.get("edit-text") {
                    edit_text
                } else {
                    DEFAULT_EDIT_TEXT
                }
            }
        };
        log::debug!("Edit Text: {}", edit_text);

        let branch = match ctx.config.get("output.html.git-branch") {
            None => "master",
            Some(toml::Value::String(b)) => b,
            _ => return Ok(book),
        };
        log::debug!("Git Branch: {}", branch);

        let mut res = None;
        book.for_each_mut(|item: &mut BookItem| {
            if let Some(Err(_)) = res {
                return;
            }

            if let BookItem::Chapter(ref mut chapter) = *item {
                res = Some(
                    open_on(
                        &git_root,
                        &src_root,
                        &repository_url,
                        &branch,
                        link_text,
                        edit_text,
                        &source_control_host,
                        chapter,
                    )
                    .map(|md| {
                        chapter.content = md;
                    }),
                );
            }
        });

        res.unwrap_or(Ok(())).map(|_| book)
    }
}

fn repo_url_to_host(repository_url: &str) -> Option<SourceControlHost> {
    if repository_url.find("github.com").is_some() {
        Some(SourceControlHost::GitHub)
    } else if repository_url.find("gitlab.com").is_some() {
        Some(SourceControlHost::GitLab)
    } else {
        None
    }
}

fn open_on(
    git_root: &Path,
    src_root: &Path,
    base_url: &str,
    branch: &str,
    link_text: &str,
    edit_text: &str,
    source_control_host: &SourceControlHost,
    chapter: &mut Chapter,
) -> Result<String> {
    let content = &chapter.content;

    let footer_start = "<footer id=\"open-git-repo\">";
    if content.contains(footer_start) {
        return Ok(content.into());
    }

    let path = match chapter.path.as_ref() {
        None => return Ok("".into()),
        Some(path) => path,
    };
    let path = match src_root.join(&path).canonicalize() {
        Ok(path) => path,
        Err(_) => return Ok(content.into()),
    };
    let relpath = path.strip_prefix(git_root).unwrap();
    log::trace!("Chapter path: {}", path.display());
    log::trace!("Relative path: {}", relpath.display());

    let edit_fragment = host_to_edit_uri_fragment(source_control_host);
    let url = format!(
        "{}/{}/{}/{}",
        base_url,
        edit_fragment,
        branch,
        relpath.display()
    );
    log::trace!("URL: {}", url);
    let link = format!("<a href=\"{}\">{}</a>", url, link_text);
    let content = format!(
        "{}\n{}{}{}</footer>",
        content, footer_start, edit_text, link
    );

    Ok(content)
}

fn host_to_edit_uri_fragment(source_control_host: &SourceControlHost) -> &str {
    match source_control_host {
        SourceControlHost::GitHub => "edit",
        SourceControlHost::GitLab => "-/edit",
    }
}

fn find_git(path: &Path) -> Option<PathBuf> {
    let mut current_path = path;
    let mut git_dir = current_path.join(".git");
    let root = Path::new("/");

    while !git_dir.exists() {
        current_path = match current_path.parent() {
            Some(p) => p,
            None => return None,
        };

        if current_path == root {
            return None;
        }

        git_dir = current_path.join(".git");
    }

    git_dir.parent().map(|p| p.to_owned())
}
