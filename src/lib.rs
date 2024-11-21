#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use clap::Command;
use dialoguer::{Confirm, Input, Select};
use serde_json::json;
use std::fs;
use std::process;

#[derive(Debug)]
struct ProjectOptions {
  project_name: String,
  strictness: Strictness,
  is_transpiler: bool,
  is_library: bool,
  is_monorepo: bool,
  is_dom: bool,
}

#[derive(Debug)]
enum Strictness {
  Off,
  On,
  Strict,
}

#[napi]
pub fn run() {
  let _cmd = Command::new("tsconfig-init")
    .about("Initialize a TypeScript project")
    .get_matches();

  match init() {
    Ok(_) => (),
    Err(e) => {
      eprintln!("Error: {}", e);
      process::exit(1);
    }
  }
}

fn init() -> Result<(), Box<dyn std::error::Error>> {
  let options = prompt_options()?;

  let project_dir = if options.project_name == "." {
    std::env::current_dir()?
  } else {
    let current_dir = std::env::current_dir()?;
    current_dir.join(&options.project_name)
  };

  fs::create_dir_all(&project_dir)?;

  let tsconfig = generate_tsconfig(&options);
  let tsconfig_path = project_dir.join("tsconfig.json");
  fs::write(&tsconfig_path, serde_json::to_string_pretty(&tsconfig)?)?;

  println!(
    "tsconfig.json has been generated in {}",
    project_dir.display()
  );
  Ok(())
}

fn prompt_options() -> Result<ProjectOptions, Box<dyn std::error::Error>> {
  let project_name = Input::<String>::new()
    .with_prompt("What is the name of your project?")
    .default(".".into())
    .interact()?;

  let strictness_options = &[
    "Relaxed (Few checks)",
    "Balanced (Recommended)",
    "Rigorous (Maximum safety)",
  ];
  let strictness_idx = Select::new()
    .with_prompt("How strict should the typescript compiler be?")
    .default(1)
    .items(strictness_options)
    .interact()?;

  let strictness = match strictness_idx {
    0 => Strictness::Off,
    1 => Strictness::On,
    2 => Strictness::Strict,
    _ => unreachable!(),
  };

  let is_transpiler = Confirm::new()
    .with_prompt("Are you transpiling using tsc?")
    .default(true)
    .interact()?;

  let is_library = Confirm::new()
    .with_prompt("Are you building a library?")
    .default(false)
    .interact()?;

  let is_monorepo = Confirm::new()
    .with_prompt("Are you building for a library in a monorepo?")
    .default(false)
    .interact()?;

  let is_dom = Confirm::new()
    .with_prompt("Is your project for a dom (browser) environment?")
    .default(false)
    .interact()?;

  Ok(ProjectOptions {
    project_name,
    strictness,
    is_transpiler,
    is_library,
    is_monorepo,
    is_dom,
  })
}

fn generate_tsconfig(options: &ProjectOptions) -> serde_json::Value {
  let mut compiler_options = json!({
      "esModuleInterop": true,
      "skipLibCheck": true,
      "target": "es2022",
      "allowJs": true,
      "resolveJsonModule": true,
      "moduleDetection": "force",
      "isolatedModules": true,
      "verbatimModuleSyntax": true,
  });

  // Strictness settings
  match options.strictness {
    Strictness::Strict => {
      compiler_options.as_object_mut().unwrap().extend(
        json!({
            "strict": true,
            "noUncheckedIndexedAccess": true,
            "noImplicitOverride": true,
        })
        .as_object()
        .unwrap()
        .clone(),
      );
    }
    Strictness::On => {
      compiler_options
        .as_object_mut()
        .unwrap()
        .insert("strict".to_string(), json!(true));
    }
    Strictness::Off => {}
  }

  // Transpiling settings
  if options.is_transpiler {
    compiler_options.as_object_mut().unwrap().extend(
      json!({
          "module": "NodeNext",
          "outDir": "dist",
          "sourceMap": true,
      })
      .as_object()
      .unwrap()
      .clone(),
    );
  } else {
    compiler_options.as_object_mut().unwrap().extend(
      json!({
          "module": "preserve",
          "noEmit": true,
      })
      .as_object()
      .unwrap()
      .clone(),
    );
  }

  // Library settings
  if options.is_library {
    compiler_options
      .as_object_mut()
      .unwrap()
      .insert("declaration".to_string(), json!(true));
  }

  // Monorepo settings
  if options.is_monorepo {
    compiler_options.as_object_mut().unwrap().extend(
      json!({
          "composite": true,
          "declarationMap": true,
      })
      .as_object()
      .unwrap()
      .clone(),
    );
  }

  // DOM settings
  if options.is_dom {
    compiler_options
      .as_object_mut()
      .unwrap()
      .insert("lib".to_string(), json!(["es2022", "dom", "dom.iterable"]));
  } else {
    compiler_options
      .as_object_mut()
      .unwrap()
      .insert("lib".to_string(), json!(["es2022"]));
  }

  json!({
      "compilerOptions": compiler_options
  })
}
