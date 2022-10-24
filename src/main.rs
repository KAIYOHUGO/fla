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
        if let syntax::Item::Pair(pair) = item {
            let start_with = pair.key.chars().next().unwrap();
            writeln!(writer, "- {} #card #start_with_{}", pair.key, start_with)?;
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
    }
    Ok(())
}

fn format(ast: syntax::Root, writer: &mut impl Write) -> io::Result<()> {
    let mut pairs: Vec<_> = ast
        .into_iter()
        .filter_map(|item| {
            if let syntax::Item::Pair(pair) = item {
                Some(pair)
            } else {
                None
            }
        })
        .collect();
    pairs.sort_by(|a, b| a.key.cmp(b.key));
    for pair in pairs {
        writeln!(writer, "{} {{", pair.key)?;
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
