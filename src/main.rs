// #![cfg_attr(feature = "dev", feature(plugin))]
// #![cfg_attr(feature = "dev", plugin(clippy))]
// #![cfg_attr(
//     feature = "dev",
//     warn(
//         cast_possible_truncation, cast_possible_wrap, cast_precision_loss, cast_sign_loss, mut_mut,
//         non_ascii_literal, result_unwrap_used, shadow_reuse, shadow_same, unicode_not_nfc,
//         wrong_self_convention, wrong_pub_self_convention
//     )
// )]
// #![cfg_attr(feature = "dev", allow(string_extend_chars))]

extern crate chrono;

#[macro_use]
extern crate failure;
extern crate semver;
extern crate serde;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[macro_use]
extern crate structopt;
// extern crate structopt_derive;

extern crate env_logger;

#[macro_use]
extern crate log;

#[macro_use]
extern crate if_chain;

extern crate alfred;
extern crate alfred_rs;
extern crate rusty_pin;

use std::borrow::Cow;
use std::env;
use std::io;
use std::process;

use alfred_rs::Data;
use alfred_rs::Updater;
use failure::Error;
use rusty_pin::Pinboard;
use structopt::StructOpt;

mod cli;
mod commands;
mod workflow_config;

use cli::{Opt, SubCommand};
use workflow_config::Config;

use commands::Runner;
// use commands::{config, delete, list, post, search, update};
use commands::config;

// TODO: add modifiers to delete commands output //
// TODO: parse Alfred preferences and get number of visible items? //

#[derive(Debug, Fail)]
pub enum AlfredError {
    #[fail(display = "Config file may be corrupted")]
    ConfigFileErr,
    #[fail(display = "Missing config file (did you set API token?)")]
    MissingConfigFile,
    #[fail(display = "What did you do?")]
    Other,
}

fn main() {
    env::set_var("alfred_workflow_data", "/Volumes/Home/hamid/tmp/rust");
    env::set_var("alfred_workflow_cache", "/Volumes/Home/hamid/tmp/rust");
    env::set_var("alfred_workflow_uid", "hamid63");
    env::set_var("alfred_workflow_name", "alfred-pinboard-rs");
    env::set_var("alfred_version", "3.6");
    env::set_var("RUST_LOG", "rusty_pin=debug,alfred_pinboard_rs=debug");
    // If user has Alfred's debug panel open, print all debug info
    // by setting RUST_LOG environment variable.
    if alfred::env::is_debug() {
        env::set_var("RUST_LOG", "rusty_pin=debug,alfred_pinboard_rs=debug");
        eprintln!("Set var RUST_LOG to: {:?}", env::var("RUST_LOG"));
    }

    env_logger::init();

    debug!("Parsing input arguments.");
    let opt: Opt = Opt::from_args();

    let pinboard;
    let config;
    debug!("Deciding on which command branch");
    match opt.cmd {
        SubCommand::Config { .. } => config::run(opt.cmd),
        _ => {
            // If user is not configuring, we will abort upon any errors.
            let s = setup().unwrap_or_else(|err| {
                show_error_alfred(err.to_string());
                process::exit(1);
            });

            let mut updater = Updater::gh("spamwax/alfred-pinboard-rs").unwrap();
            updater.set_version("0.13.1");
            updater.set_interval(60);
            updater.init().unwrap();

            pinboard = s.1;
            config = s.0;
            let mut runner = Runner {
                config: Some(config),
                pinboard: Some(pinboard),
                updater: Some(updater),
            };
            match opt.cmd {
                SubCommand::Update => {
                    runner.update_cache();
                }
                SubCommand::List { .. } => {
                    runner.list(opt.cmd);
                }
                SubCommand::Search { .. } => {
                    runner.search(opt.cmd);
                }
                SubCommand::Post { .. } => {
                    runner.post(opt.cmd);
                }
                SubCommand::Delete { .. } => {
                    runner.delete(opt.cmd);
                }
                _ => unimplemented!(),
            }
        }
    }
}

fn setup<'a, 'p>() -> Result<(Config, Pinboard<'a, 'p>), Error> {
    debug!("Starting in setup");
    let config = Config::setup()?;
    let mut pinboard = Pinboard::new(config.auth_token.clone(), alfred::env::workflow_cache())?;
    pinboard.enable_fuzzy_search(config.fuzzy_search);
    pinboard.enable_tag_only_search(config.tag_only_search);
    pinboard.enable_private_new_pin(config.private_new_pin);
    pinboard.enable_toread_new_pin(config.toread_new_pin);

    Ok((config, pinboard))
}

fn show_error_alfred<'a, T: Into<Cow<'a, str>>>(s: T) {
    debug!("Starting in show_error_alfred");
    let item = alfred::ItemBuilder::new("Error")
        .subtitle(s)
        .icon_path("erroricon.icns")
        .into_item();
    alfred::json::write_items(io::stdout(), &[item]).expect("Can't write to stdout");
}

fn alfred_error<'a, T: Into<Cow<'a, str>>>(s: T) -> alfred::Item<'a> {
    debug!("Starting in alfred_error");
    alfred::ItemBuilder::new("Error")
        .subtitle(s)
        .icon_path("erroricon.icns")
        .into_item()
}
