use base64::{engine::general_purpose, Engine as _};
use epub::doc::EpubDoc;
use std::io::Read;
use std::io::Write;
use walkdir::WalkDir;

struct Book {
    title: String,
    author: String,
    path: String,
    cover: Vec<u8>,
}

fn get_book_data(root_dir: &str) -> Vec<Book> {
    let mut books: Vec<Book> = Vec::new();

    for entry in WalkDir::new(root_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "epub") {
            match EpubDoc::new(path.to_str().unwrap()) {
                Ok(mut epub) => {
                    let mut book = Book {
                        title: "".to_string(),
                        author: "".to_string(),
                        path: "".to_string(),
                        cover: Vec::new(),
                    };

                    book.path = path
                        .to_str()
                        .unwrap()
                        .to_string()
                        .strip_prefix(root_dir)
                        .unwrap()
                        .to_string();
                    book.title = epub.mdata("title").unwrap_or("N.A.".to_string());
                    book.author = epub.mdata("creator").unwrap_or("N.A.".to_string());

                    match epub.get_cover() {
                        Some(cover) => book.cover = cover.0,
                        None => {
                            println!("Cover: None");
                        }
                    }

                    print_book(&book);
                    books.push(book)
                }
                Err(err) => {
                    eprintln!("Error opening EPUB file {}: {}", path.display(), err);
                }
            }
        }
    }

    books
}

fn print_book(book: &Book) {
    println!(
        "{} - {} - {} - {} bytes of cover img",
        book.author,
        book.title,
        book.path,
        book.cover.len()
    )
}

fn books_to_html(books: &Vec<Book>) -> String {
    let style = r#"
    <style>
    .container{
        width: 100%;
        font-family: RobotoSlab,sans-serif;
    }
    th {
        font-size: 1.5rem;
        line-height: 2rem;
        font-weight: 700;
        top: 0;
        position: sticky;
        background: white;
    }
    tr {
        border-top-width: 1px;
        border-top-color: #cccccc;
    }
    img {
        max-width: 24rem;
        max-height: 24rem;
    }
    table {
        width: 100%;
    }
    td {
        padding: 0.5rem;
        word-break: normal;
        overflow-wrap: normal;
        max-width: 20%;
    }
    .text-left {
        text-align: left;
    }
    </style>
    "#;

    let mut table = String::new();

    table.push_str(
        r#"
    <table>
    <thead><tr>
        <th>Cover</th>
        <th class="text-left">Title</th>
        <th class="text-left">Author</th>
        <th class="text-left">Path</th>
    </tr></thead>
    "#,
    );

    table.push_str("<tbody>\n");
    for book in books {
        table.push_str("<tr class=\"border-t\">");
        table.push_str(&format!(
            "<td><center><img src=\"data:image/png;base64,{}\" /></center></td>",
            general_purpose::STANDARD.encode(&book.cover)
        ));
        table.push_str(&format!("<td>{}</td>", book.title));
        table.push_str(&format!("<td>{}</td>", book.author));
        table.push_str(&format!("<td>{}</td>", book.path));
        table.push_str("</tr>\n");
    }
    table.push_str("</tbody></table>");

    format!(
        r#"
        <html>
            <head>{}</head>
            <body>
                <div class="container">{}</div>
            </body>
        </html>
        "#,
        style, table
    )
}

fn main() {
    let directory = ".";

    let mut books = get_book_data(directory);

    println!();
    println!();
    println!();
    println!("Read {} epub files", books.len());
    println!("Sorting for author");
    books.sort_by_key(|b| b.author.clone());

    println!("Creating HTML output");
    let html = books_to_html(&books);

    println!("Writing HTML output to file, do not close!");
    let mut file = std::fs::File::create("epub-index.html").expect("cannot open html file");
    file.write_all(html.as_bytes()).expect("cannot write html");

    println!();
    println!();
    println!();
    println!("Wrote index to epub-index.html");
    println!("Press ENTER to quit...");
    let mut buffer = [0; 1];
    std::io::stdin()
        .read_exact(&mut buffer)
        .expect("Failed to read input");
}
