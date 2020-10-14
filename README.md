# mdbook-open-git-repo

A preprocessor for [mdbook][] to add an edit link on every page. Forked and adapted from
[mdbook-open-on-gh][] to add support for other platforms than github.com such as a
self-hosted GitLab instance.

[mdbook]: https://github.com/rust-lang/mdBook
[mdbook-open-on-gh]: https://github.com/badboy/mdbook-open-on-gh

It adds a customisable link (default: "Edit this file on GitHub.") on the bottom
of every page, linking directly to the source file. It uses the configured
`git-repository-url` as the base.

## Installation

At the current time, this repo is not published on crates.io. Grab a binary from the release page
and place it somewhere in your `$PATH`.

Add it as a preprocessor to your `book.toml`:

```
[preprocessor.open-git-repo]
command = "mdbook-open-git-repo"
renderer = ["html"]
```

Add a repository URL to use as a base in your `book.toml`:

```toml
[output.html]
git-repository-url = "https://github.com/organization/repo"
```

Repos hosted on either github.com or gitlab.com will automatically be detected.
If you are using eg. a self-hosted GitLab instance, you must specify the type
of host:

```toml
[preprocessor.open-git-repo]
source-control-host = "gitlab"
```

The default content generated is the following:

```
Found a bug? <a href="https://git.host.com/repo/project/branch/edit/chapter.md">Edit this file on GitHub.</a>
```

Both the text before and inside the `<a>` tag may be customised:

```toml
[preprocessor.open-git-repo]
edit-text = "Something not right?"
link-text = "Fix it immediately!"
```

Which would produce the following output:

```
Something not right? <a href="/link/to/edit/page">Fix it immediately!</a>
```


To style the footer add a custom CSS file for your HTML output:

```toml
[output.html]
additional-css = ["open-in.css"]
```

And in `open-in.css` style the `<footer>` element or directly the CSS element id `open-git-repo`:

```css
footer {
  font-size: 0.8em;
  text-align: center;
  border-top: 1px solid black;
  padding: 5px 0;
}
```

This code block shrinks the text size, center-aligns it under the rest of the content
and adds a small horizontal bar above the text to separate it from the page content.


Finally, build your book as normal:

```
mdbook path/to/book
```

## License

MPL. See [LICENSE](LICENSE).  
Original code copyright (c) 2020 Jan-Erik Rediger <janerik@fnordig.de>.
