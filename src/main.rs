use clap::Parser;
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::{fs, io};
use wildmatch::WildMatch;

fn get_ignore_list(ignore_file_path: &Path, output_file_name: &str) -> Vec<String> {
    let mut ignore_list = vec![];

    if let Ok(ignore_file) = fs::read_to_string(ignore_file_path) {
        for line in ignore_file.lines() {
            let line = if cfg!(windows) {
                line.replace("/", "\\")
            } else {
                line.to_string()
            };
            ignore_list.push(line);
        }
    }

    // Add the output file to the ignore list.
    ignore_list.push(output_file_name.to_string());

    ignore_list
}

fn should_ignore(file_path: &Path, ignore_list: &[String]) -> bool {
    ignore_list.iter().any(|pattern| {
        let wm = WildMatch::new(pattern);
        wm.matches(file_path.to_str().unwrap())
    })
}

fn process_repository(
    repo_path: &Path,
    ignore_list: &[String],
    output_writer: &mut Box<dyn Write>,
) {
    for entry in globwalk::GlobWalkerBuilder::new(repo_path, "**/*")
        .file_type(globwalk::FileType::FILE)
        .build()
        .expect("Failed to build glob iterator")
        .into_iter()
        .filter_map(Result::ok)
    {
        let file_path = entry.path();
        let relative_file_path = file_path.strip_prefix(repo_path).unwrap();

        if !should_ignore(relative_file_path, ignore_list) {
            if let Ok(mut file) = fs::File::open(&file_path) {
                let mut contents = Vec::new();
                file.read_to_end(&mut contents)
                    .expect("Cannot read file contents");

                // Decode the contents as UTF-8, replacing invalid sequences
                let contents_str = String::from_utf8_lossy(&contents);

                writeln!(output_writer, "----").expect("Cannot write to output file");
                writeln!(output_writer, "{}", relative_file_path.display())
                    .expect("Cannot write to output file");
                writeln!(output_writer, "{}", contents_str).expect("Cannot write to output file");
            }
        }
    }
}

#[derive(Parser)]
#[clap(name = "Yeet a Git repository into a text file.")]
struct Opts {
    /// The path to the Git repository.
    repo_path: String,

    /// The path to the preamble file. If not given, uses a sensible default.
    #[clap(short = 'p', long = "preamble")]
    preamble: Option<String>,

    /// The path to the output file.
    #[clap(short = 'o', long = "output", default_value = "output.txt")]
    output: String,

    /// The path to a custom ignore file. If not given, tries to fallback to `{repo_path}/.gptignore`.
    #[clap(short = 'i', long = "ignore")]
    ignore: Option<String>,

    /// Print repository contents to stdout.
    #[clap(short = 's', long = "stdout", conflicts_with = "output")]
    print_to_stdout: bool,

    /// Print repository contents to stderr.
    #[clap(short = 'r', long = "stderr", conflicts_with = "output")]
    print_to_stderr: bool,
}

fn main() {
    let opts: Opts = Opts::parse();

    let repo_path = Path::new(&opts.repo_path);
    let mut ignore_file_path = if let Some(ignore) = opts.ignore {
        PathBuf::from(ignore)
    } else {
        repo_path.join(".gptignore")
    };

    if cfg!(windows) {
        ignore_file_path = PathBuf::from(ignore_file_path.to_str().unwrap().replace("/", "\\"));
    }

    let output_file_name = Path::new(&opts.output)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();

    let ignore_list = if ignore_file_path.exists() {
        get_ignore_list(&ignore_file_path, output_file_name)
    } else {
        vec![output_file_name.to_string()] // Add the output file to the ignore list even if .gptignore doesn't exist
    };

    let mut output_writer: Box<dyn Write> = if opts.print_to_stdout {
        Box::new(BufWriter::new(io::stdout()))
    } else if opts.print_to_stderr {
        Box::new(BufWriter::new(io::stderr()))
    } else {
        Box::new(fs::File::create(&opts.output).unwrap())
    };

    if let Some(preamble_file) = opts.preamble {
        let preamble_text = fs::read_to_string(preamble_file).unwrap();
        writeln!(output_writer, "{}", preamble_text).unwrap();
    } else {
        writeln!(output_writer, "The following text is a Git repository with code. The structure of the text are sections that begin with ----, followed by a single line containing the file path and file name, followed by a variable amount of lines containing the file contents. The text representing the Git repository ends when the symbols --END-- are encounted. Any further text beyond --END-- are meant to be interpreted as instructions using the aforementioned Git repository as context.").unwrap();
    }

    process_repository(&repo_path, &ignore_list, &mut output_writer);

    writeln!(output_writer, "--END--").unwrap();

    if !opts.print_to_stderr & !opts.print_to_stdout {
        println!("Repository contents written to {}.", opts.output);
    }
}
