use std::{
    error::Error,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use comrak::{markdown_to_html, ComrakOptions};

pub fn finalize_dist(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    //Overwrite dist dir if already exists or create
    if Path::new("./dist").exists() {
        fs::remove_dir_all("./dist").unwrap();
    }

    fs::create_dir("./dist")?;

    //Goes through each arg of input
    args.into_iter().skip(1).for_each(|file| {
        //If it is a directory it recursively gets all files and places them in dist
        if Path::new(&file).is_dir() {
            recursive(file.into());
        } else {
            //Otherwise basic file movement to dist
            //Quick Check to see if file extension is .txt or .md
            if file.contains(".txt") {
                let html = file.strip_suffix(".txt").unwrap().to_owned() + ".html";
                let new_location = "./dist/".to_owned() + &html;
                fs::copy(&html, new_location).unwrap();
                fs::remove_file(html).unwrap();
            } else {
                let md = file.strip_suffix(".md").unwrap().to_owned() + ".html";
                let new_location = "./dist/".to_owned() + &md;
                fs::copy(&md, new_location).unwrap();
                fs::remove_file(md).unwrap();
            }
        }
    });

    Ok(())
}

pub fn process_arguments(args: &[String]) -> Result<(), Box<dyn Error>> {
    //Iterate through each input and process
    args.iter().for_each(|arg| {
        if let Ok(mut file) = fs::read_to_string(arg.to_owned()) {
            //Reaches this if the argument was just a filename
            process(&mut file, arg);
        } else if Path::new(arg).is_dir() {
            //Argument is a directory so we have to recursively search the dir
            let path = Path::new(arg);
            visit_dirs(path, &process).expect("Couldn't convert dir");
        }
    });

    Ok(())
}

//recursively reads the directory and passes all files to the proccess function to be turned to html
fn visit_dirs(dir: &Path, cb: &dyn Fn(&mut String, &str)) -> Result<(), Box<dyn Error>> {
    //Checks to make sure it is a directory
    if dir.is_dir() {
        //Reads all entries (files and directories) in this curr directory
        for entry in fs::read_dir(dir)? {
            //Unwraps entry down to path
            let entry = entry?;
            let path = entry.path();

            //If the path is another directory recursively call visit_dirs
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else if let Ok(mut file) = fs::read_to_string(&path) {
                //Not a directory so call the proccess function which was passed as callback function
                let pathname = path.to_string_lossy();
                let filename = pathname.strip_prefix(dir.to_str().unwrap()).unwrap();
                cb(&mut file, filename);
            }
        }
    }
    Ok(())
}

fn process(file: &mut String, filename: &str) {
    //Create name array containing filname string array
    let mut name = filename.to_string();
    let mut ext = "";

    //Check to see if the filename contains extension .txt or .md
    if filename.contains(".txt") {
        //Create final file name: test.txt -> test.html
        name = name.replace(".txt", ".html");
        ext = "txt";
    } else if filename.contains(".md") {
        name = name.replace(".md", ".html");
        ext = "md";
    }

    //When doing nested subdirectories a / would left from the subirectory name ex. /test.html
    if name.starts_with('/') {
        name = name[1..].to_string();
    }

    //This will track if I need to create a new paragraph tag for soft newlines
    let mut firstline = true;

    //Sets the rules for the html files created so that lines can be appended
    let mut html = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&name)
        .unwrap();

    //Collects all lines of txt file into vec of strings to iterate line by line
    let vec_lines: Vec<&str> = file.lines().into_iter().collect();
    let mut line = String::new();
    let prev_tag = "";

    //Default content (content that is always the same) will be added throughout proccess
    let default_content = "<!doctype html>
<html lang=\"en\">
<head>
\t<meta charset=\"utf-8\">";

    html.write_all(default_content.as_bytes()).unwrap();

    //Checks if there is a title. If there is does proccessing based as such otherwise regular
    //processing
    if vec_lines[1].is_empty() && vec_lines[2].is_empty() && !vec_lines[0].is_empty() {
        process_file_with_title(vec_lines, &mut html, &mut firstline, &mut line, prev_tag, ext);
    } else {
        process_file_with_no_title(
            &name,
            default_content,
            vec_lines,
            &mut html,
            &mut firstline,
            &mut line,
            prev_tag,
            ext,
        )
    }

    let default_content;
    if ext == "txt" {
        default_content = "</p>
</body>
</html>";
    } else {
        default_content = "
</body>
</html>";
    }

    html.write_all(default_content.as_bytes()).unwrap();
}

//recursively gets the correct file name to place in dist folder
fn recursive(dir: PathBuf) {
    //Get directory name/path
    let dir_name = dir.to_str().unwrap();
    for entry in fs::read_dir(dir_name).unwrap() {
        //Go through the dir
        if entry.as_ref().unwrap().path().is_dir() {
            //If another dir is found recursive call
            recursive(entry.unwrap().path());
        } else {
            //It is a file so proccess it from txt to html
            let mut html = entry
                .unwrap()
                .path()
                .to_string_lossy()
                .to_string()
                .strip_prefix(&dir_name)
                .unwrap()
                .strip_suffix(".txt")
                .unwrap()
                .to_owned()
                + ".html";

            //Subdirectory files contain a / at the start -> /test.txt
            if html.starts_with('/') {
                html = html[1..].to_string();
            }

            //Place in dist
            let new_location = "./dist/".to_owned() + &html;
            fs::copy(&html, &new_location).unwrap();
            fs::remove_file(html).unwrap();
        }
    }
}

fn process_file_with_title(
    vec_lines: Vec<&str>,
    html: &mut File,
    firstline: &mut bool,
    line: &mut String,
    mut prev_tag: &str,
    ext: &str,
) {
    //Write title
    let mut title_name = vec_lines[0];

    //Check to see if title contains bold markdown syntax
    if vec_lines[0].contains("# ") {
        title_name = title_name.strip_prefix("# ").unwrap();
    }

    let default_content = &("\n\t<title>".to_owned() + title_name + "</title>");
    html.write_all(default_content.as_bytes()).unwrap();

    let default_content = "
\t<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">
</head>
<body>\n";

    html.write_all(default_content.as_bytes()).unwrap();
    //H1 with the title
    html.write_all(("\t<h1>".to_owned() + title_name + "</h1>\n\n").as_bytes())
        .unwrap();

    if ext == "md" {
        let file: Vec<&str> = vec_lines.into_iter().skip(3).collect();
        let file = file.join("\n");
        let file = markdown_to_html(&file, &ComrakOptions::default());
        html.write_all(file.as_bytes()).expect("Could not write markdown");
        return;
    }

    //Skip first 3 lines as it is title info
    vec_lines.into_iter().skip(3).for_each(|curr_line| {
        //If the line isn't empty it is part of a p tag
        if !curr_line.is_empty() {
            //Checks if it is the first line of paragraph
            if *firstline {
                //If so the we can print check for --- or print the opening tag and set firstline as false
                *line = "\t<p>".to_owned() + curr_line;
                prev_tag = "<p>";
                *firstline = false;
            } else {
                //We can then print other lines of the paragraph as regular lines if prev tag was a paragraph
                *line = "\n\t".to_owned() + curr_line;
            }

            html.write_all(line.as_bytes()).expect("Could not write to file");
        } else {
            //This means there was a hard newline since line is empty so we print the closing p tag
            //for prev paragraph and set firstline as true for the next paragraph
            *firstline = true;

            if prev_tag == "<p>" {
                html.write_all("</p>\n\n".as_bytes()).expect("Could not write to file");
            } else {
                html.write_all("\n\n".as_bytes()).expect("Could not write to file");
            }
        }
    });
}

fn process_file_with_no_title(
    name: &str,
    default_content: &str,
    vec_lines: Vec<&str>,
    html: &mut File,
    firstline: &mut bool,
    line: &mut String,
    mut prev_tag: &str,
    ext: &str,
) {
    //Same logic but without the title tag in head
    html.write_all(default_content.as_bytes()).unwrap();
    //Title instead of h1
    let default_content = &("\n\t<title>".to_owned() + name + "</title>");
    html.write_all(default_content.as_bytes()).unwrap();

    let default_content = "
\t<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">
</head>
<body>\n";

    html.write_all(default_content.as_bytes()).unwrap();

    if ext == "md" {
        let file: Vec<&str> = vec_lines.into_iter().collect();
        let file = file.join("\n");
        let file = markdown_to_html(&file, &ComrakOptions::default());
        html.write_all(file.as_bytes()).expect("Could not write markdown");
        return;
    }

    vec_lines.into_iter().for_each(|curr_line| {
        if !curr_line.is_empty() {
            //Checks if it is the first line of paragraph
            if *firstline {
                //If so the we can print the opening tag and set firstline as false
                *line = "\t<p>".to_owned() + curr_line;
                prev_tag = "<p>";
                *firstline = false;
            } else {
                //We can then print other lines of the paragraph as regular lines if prev tag was a paragraph
                *line = "\n\t".to_owned() + curr_line;
            }

            html.write_all(line.as_bytes()).expect("Could not write to file");
        } else {
            //This means there was a hard newline since line is empty so we print the closing p tag
            //for prev paragraph and set firstline as true for the next paragraph
            *firstline = true;
            if prev_tag == "<p>" {
                html.write_all("</p>\n\n".as_bytes()).expect("Could not write to file");
            } else {
                html.write_all("\n\n".as_bytes()).expect("Could not write to file");
            }
        }
    });
}
