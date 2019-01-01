extern crate scraper;
#[macro_use]
extern crate text_io;
extern crate clap;

use std::fs;

use clap::{App, Arg};
use scraper::{Html, Selector};
use std::io;
use std::path::Path;

const GITHUB_EVENTS_DOC_URL: &str = "https://developer.github.com/v3/activity/events/types/";
const SETTING_OUTPUT_DIR: &str = "output_dir";

/// Retrieve the HTML of the GitHub docs page for Events.
fn get_html_doc() -> Result<Html, reqwest::Error> {
    let body = reqwest::get(GITHUB_EVENTS_DOC_URL)?.text()?;
    let html = Html::parse_document(body.as_str());
    Ok(html)
}

/// Extract each codeblock with its heading.
fn get_heading_and_codeblock() -> Vec<(String, String)> {
    // Prepare HTML document for parsing.
    let html_doc = get_html_doc().unwrap();
    let selector = Selector::parse("h2, h3 ~ pre > code").unwrap();

    // A place to stash the heading as they are encountered.
    let mut heading = "";

    // Gather up heading / codeblock pairs (to be returned).
    let mut pairings = Vec::new();

    for element in html_doc.select(&selector) {
        let element_name = element.value().name();
        if element_name == "h2" {
            let h2 = element.text().collect::<Vec<_>>();

            // Skip over h2 elements that don't look like what we're after.
            if h2[0] != "\n" {
                continue;
            }

            heading = h2[1];
        } else if element_name == "code" {
            println!("heading: {}", heading);
            pairings.push((// Gather pairings to be exported.
                           // Heading of the code block which correlates to the webhook Event Name.
                           heading.to_string(),
                           // Text of the code block containing the JSON payload.
                           element.text().collect::<Vec<_>>().join("")))
        } else {
            panic!("Encountered an unsupported HTML element: {}", element_name)
        }
    }

    pairings
}

/// Prepare the directory that will receive the GitHub Event JSON files.
fn prepare_output_dir(output_dir: &str) -> Result<(), io::Error> {
    // If export files already exist, give the User a change to:
    //   [D]elete delete all files in the directory $path
    //   [L]eave all files in $path/, but also over-write any JSON files with the same name.
    if Path::new(output_dir).exists() && fs::read_dir(output_dir)?.collect::<Vec<_>>().len() > 0 {
        loop {
            println!("Files exist in `{}`! [D]elete them or [L]eave them alone?",
                     output_dir);
            let user_input: String = read!("{}\n");
            match user_input.to_lowercase().as_str() {
                "d" => {
                    fs::remove_dir_all(output_dir)?;
                    break;
                }
                "l" => break,
                _ => continue,
            }
        }
    }

    fs::create_dir_all(output_dir)
}

/// Write `event_json` to `event_name`.json
fn write_event(output_dir: &str, event_name: &str, event_json: &String) -> Result<(), io::Error> {
    let path = format!("{}/{}.json", output_dir, event_name);
    println!("Writing Event JSON {}", event_name);
    fs::write(path, event_json)
}

fn main() {
    let args = App::new("GitHubEventTypePayloadScraper")
        .version("1.0")
        .author("Lyle Scott, III <lyle@ls3.io>")
        .about("Scrapes GitHub Event type payloads")
        .arg(
            Arg::with_name("output_dir")
                .help("Directory to output JSON files to")
                .required(true)
                .takes_value(true)
                .short("o")
                .long("output_dir"),
        )
        .get_matches();

    let heading_codeblock_pairs = get_heading_and_codeblock();
    let output_dir = args.value_of(SETTING_OUTPUT_DIR).unwrap();
    prepare_output_dir(output_dir).unwrap();
    for (event_name, event_json) in &heading_codeblock_pairs {
        write_event(output_dir, event_name, event_json).unwrap();
    }
}
