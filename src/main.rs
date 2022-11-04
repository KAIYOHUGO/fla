use clap::{Parser, ValueEnum};
use std::{
    error::Error,
    fs::File,
    io::{self, Read, Write},
    path::PathBuf,
};
mod syntax;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct CLi {
    #[arg(value_enum)]
    mode: RunType,

    input: PathBuf,

    #[clap(short, long)]
    output: Option<PathBuf>,
}

#[derive(Debug, Clone, ValueEnum)]
enum RunType {
    Build,
    Debug,
    Fmt,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = CLi::parse();

    let mut s = String::new();
    let ast = {
        let mut f = File::open(&cli.input)?;
        f.read_to_string(&mut s).unwrap();
        syntax::parse(s.as_str())?
    };

    match cli.mode {
        RunType::Build => {
            let mut output = File::create(
                cli.output
                    .unwrap_or_else(|| cli.input.with_extension("fla.md")),
            )?;
            build(ast, &mut output)?
        }
        RunType::Debug => {
            let mut output = File::create(
                cli.output
                    .unwrap_or_else(|| cli.input.with_extension("fla.debug")),
            )?;
            write!(&mut output, "{:?}", ast)?
        }
        RunType::Fmt => format(ast, &mut File::create(cli.output.unwrap_or(cli.input))?)?,
    }
    Ok(())
}

fn build(ast: syntax::Root, writer: &mut impl Write) -> io::Result<()> {
    for item in ast {
        let syntax::Item::Pair(pair) = item;

        let start_with = match pair.key.first().unwrap() {
            syntax::Key::Text(s) | syntax::Key::Cloze(s) => s.chars().next().unwrap(),
        };

        let key: String = pair
            .key
            .into_iter()
            .map(|x| match x {
                syntax::Key::Text(s) => s.into(),
                syntax::Key::Cloze(s) => format!(" {{{{cloze {}}}}} ", s),
            })
            .collect();
        writeln!(writer, "- {} #card #start_with_{}", key, start_with)?;
        for value in pair.value {
            match value {
                syntax::Value::Node(node) => {
                    let mut lines = node.text.lines().map(|s| s.trim());
                    writeln!(
                        writer,
                        "    - =={}.== {}",
                        node.speech,
                        lines.next().unwrap()
                    )?;
                    for line in lines {
                        writeln!(writer, "      {}", line)?;
                    }
                }
                syntax::Value::Text(text) => {
                    let mut lines = text.lines().map(|s| s.trim());
                    writeln!(writer, "    - {}", lines.next().unwrap())?;
                    for line in lines {
                        writeln!(writer, "      {}", line)?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn format(ast: syntax::Root, writer: &mut impl Write) -> io::Result<()> {
    let mut pairs: Vec<_> = ast
        .into_iter()
        .filter_map(|item| {
            let syntax::Item::Pair(pair) = item;
            Some(pair)
        })
        .collect();
    pairs.sort_by(|a, b| a.key.cmp(&b.key));
    for pair in pairs {
        let key: String = {
            pair.key
                .into_iter()
                .map(|x| match x {
                    syntax::Key::Text(s) => s.into(),
                    syntax::Key::Cloze(s) => format!(" {{{{{}}}}} ", s),
                })
                .collect()
        };

        writeln!(writer, "{} {{", key)?;
        for value in pair.value {
            match value {
                syntax::Value::Node(node) => {
                    writeln!(writer, "    {} {{", node.speech)?;
                    for line in node.text.lines().map(|s| s.trim()) {
                        writeln!(writer, "        {}", line)?;
                    }
                    writeln!(writer, "    }}")?;
                }
                syntax::Value::Text(text) => {
                    for line in text.lines().map(|s| s.trim()) {
                        writeln!(writer, "    {}", line)?;
                    }
                }
            }
        }
        writeln!(writer, "}}")?;
        writeln!(writer)?;
    }
    Ok(())
}
