use anyhow::Result;
use glob::glob;
use orgize::{Headline, Org};
use std::path::{Path, PathBuf};
use wire::state::TodoPrelim;

fn get_org_files_in_dir(dir: &Path) -> Vec<PathBuf> {
    glob(dir.join("**/*.org").to_str().unwrap())
        .unwrap()
        .filter_map(Result::ok)
        .collect()
}

fn get_todos_from_org_file(org_file: &Path) -> Result<Vec<TodoPrelim>> {
    let contents = std::fs::read_to_string(org_file)?;
    // let mut todos = Vec::new();

    let org_parse = Org::parse_custom(
        &contents,
        &orgize::ParseConfig {
            todo_keywords: (
                vec![String::from("TODO")],
                vec![String::from("DONE"), String::from("NOT_DONE")],
            ),
        },
    );

    let todos = get_todos_from_org_parse(&org_parse);

    todo!()
}

fn get_todos_from_org_parse(org_parse: &Org) -> Vec<Result<TodoPrelim>> {
    // println!(
    //     "{:#?}",
    //     org_parse
    //         .iter()
    //         .filter_map(|el| match el {
    //             orgize::Event::Start(el) => Some(el),
    //             orgize::Event::End(_) => None,
    //         })
    //         .collect::<Vec<_>>()
    // );
    org_parse
        .headlines()
        .map(|h| get_todo_from_headline(h, org_parse))
        .collect()
}

fn get_todo_from_headline<'a>(headline: Headline, org_parse: &Org) -> Result<TodoPrelim> {
    let section_node_id = headline
        .section_node()
        .ok_or(anyhow::anyhow!("No section node for headline"))?;

    headline.title(org_parse).scheduled();

    todo!()
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use orgize::Org;

    use crate::get_todos_from_org_parse;

    #[test]
    fn test_get_todos_from_org_parse() -> Result<()> {
        let org_parse = Org::parse(
            "* DONE Allow ignoring rule action logs that have been undone
  CLOSED: [2023-10-27 Fri 11:56]
* DONE Say what undo actually means in the UI.
  SCHEDULED: <2023-10-29 Sun 19:00> CLOSED: [2023-10-30 Mon 10:38]
  :LOGBOOK:
  CLOCK: [2023-10-31 Tue 14:35]
  CLOCK: [2023-10-30 Mon 08:45]--[2023-10-30 Mon 10:37] => 1:52
  :END:
* TODO Build a lightweight version of the reports system.
SCHEDULED: <2023-10-29 Sun 18:30>",
        );

        // get_todos_from_org_parse(&org_parse);
        Ok(())
    }
}
