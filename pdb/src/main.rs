use std::io;

use bat::line_range::{LineRange, LineRanges};
use bat::PrettyPrinter;
use clap::Parser;
use crossterm::{cursor, execute, queue, terminal};

use dusk_pdb::prelude::*;

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Args::parse().resolve()?;
    let mut app = App::load(args).await?;

    let config = app.config().clone();

    let mut stdout = io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen, cursor::MoveTo(0, 0))?;

    while let Some(Output {
        contents,
        console,
        error,
    }) = app.next_output().await
    {
        if let Some(Source {
            name,
            contents,
            line,
        }) = contents
        {
            queue!(
                stdout,
                terminal::Clear(terminal::ClearType::All),
                cursor::MoveTo(1, 1)
            )?;

            println!("{}", name);

            let margin = config.render.margin;
            let range = LineRanges::from(vec![LineRange::new(
                line.saturating_sub(margin),
                line.saturating_add(margin),
            )]);

            PrettyPrinter::new()
                .input_from_bytes(contents.as_bytes())
                .language("rust")
                .header(config.render.header)
                .grid(config.render.grid)
                .line_numbers(config.render.line_numbers)
                .line_ranges(range)
                .highlight(line)
                .theme(&config.render.theme)
                .print()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        }

        for error in error {
            println!("{}", error);
        }

        for console in console {
            println!("{}", console);
        }
    }

    execute!(stdout, terminal::LeaveAlternateScreen)?;

    println!("bye!");

    Ok(())
}
