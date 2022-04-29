//! # Substrate Node Template Generator
//!
//! A tool to generate stand-alone node templates of a customized Substrate clients used in
//! "Substrate Library Extension" (SLE) projects like Cumulus, Canvas, Frontier, as well as
//! any custom chain that intends for users to build off of as a base template included in their
//! source.

use fs_extra::dir::{self, CopyOptions};
use git2;
use glob;
use std::{
	collections::HashMap,
	env,
	fs::{self, File, OpenOptions},
	io::{Read, Write},
	path::{Path, PathBuf},
	process::Command,
};
use serde::Deserialize;
use toml;
use toml_edit::{Document, InlineTable, Value};
use env_logger::Env;

// Set the public repository where the stand-alone template can replace local
// cargo dependency items with.
const PROJECT_GIT_URL: &str = "https://github.com/paritytech/substrate.git";

/// Store a deserialized, parsed, configuration TOML file
#[derive(Deserialize)]
struct Config {
    upstream: Upstream,
    output: Output,
}

/// Deserialized, parsed, upstream template package information.
#[derive(Deserialize)]
struct Upstream {
    source_path: PathBuf,
    git_info: GitInfo,
    relative_template_path: PathBuf,
}

/// Deserialized, parsed, git target info
#[derive(Deserialize)]
struct GitInfo {
	url: String,
    selector: GitType,
    name: String,
}

/// Git target specification for branch OR tag OR rev
#[derive(Deserialize, Clone)]
#[allow(non_camel_case_types)]
enum GitType {
	branch,
	tag,
	rev,
}

/// Deserialized, parsed, upstream template package information.
#[derive(Deserialize)]
struct Output {
	path: PathBuf,
	overwrite: bool,
	build: bool,
	test: bool,
	zip: bool,
	package: Package,
}

/// Deserialized, parsed, `Cargo.toml` template package information to be used in the generated template.
#[derive(Deserialize)]
struct Package {
	name: String,
	authors: Vec<String>,
	description: String,
	license: String,
	homepage: String,
	repository: String,
	edition: String,
}


/// Find all `Cargo.toml` files in the given path.
fn find_cargo_tomls(path: &PathBuf) -> Vec<PathBuf> {
	let path = format!("{}/**/Cargo.toml", path.display());

	let glob = glob::glob(&path).expect("Generates globbing pattern");

	let mut result = Vec::new();
	glob.into_iter().for_each(|file| match file {
		Ok(file) => result.push(file),
		Err(e) => println!("{:?}", e),
	});

	if result.is_empty() {
		panic!("Did not found any `Cargo.toml` files.");
	}

	result
}

/// Clones the git repo from config file's `git_info.url` into
/// `source_path`
fn clone_repo(config: &Config) {
	let git_path = config.upstream.git_info.url.clone();
	let local_path = config.upstream.source_path.clone();

	let repo = match git2::Repository::clone(&git_path, local_path) {
		Ok(repo) => repo,
		Err(e) => panic!("failed to clone: {}", e),
	};
}

/// pulls and checks out the git repo at `local_path`.
fn git_pull(config: &Config) {
	// git2 library does not have a straightforward way to pull,
	// so directly using the git command to pull is an easy workaround.

	// the checkout type (branch, rev, tag)
	let git_selector = config.upstream.git_info.selector.clone();
	// the selector name (i.e. "master", or "v1.0.4", etc.)
	let git_selector_name = config.upstream.git_info.name.clone();
	let local_path = config.upstream.source_path.clone();

	//pull the git repository at `local_path`
	assert!(Command::new("git")
		.args(&["pull"])
		.current_dir(&local_path)
		.status()
		.expect("pulls git repo")
		.success());

	//git checkout based on branch or rev or tag
	assert!(Command::new("git")
		.args(&["checkout", &git_selector_name[..]]) // convert String -> slice of &str
		.current_dir(&local_path)
		.status()
		.expect("checkouts the branch | rev | tag")
		.success());
}

/// Copy the template specified to the given output path.
fn copy_node_template(config: &Config) {
	let mut options = CopyOptions::new();
	options.overwrite = config.output.overwrite;
	options.content_only =true;

	let mut abs_template_path = config.upstream.source_path.clone();
	abs_template_path.push(&config.upstream.relative_template_path);

	// Only create the folder if it doesn't exist.
	// Overwrite files within this dir if config sets this latter.
	dir::create(&config.output.path, false).expect("New node-template output dir, if doesn't exist");

	dir::copy(abs_template_path, &config.output.path , &options).expect("Copies node-template to output dir");
}

/// Gets the latest commit id of the repository given by `path`.
fn get_git_commit_id(path: &Path) -> String {
	let repo = git2::Repository::discover(path)
		.expect(&format!("Node template ({}) should be in a git repository.", path.display()));

	let commit_id = repo
		.head()
		.expect("Repository should have a head")
		.peel_to_commit()
		.expect("Head references a commit")
		.id();

	format!("{}", commit_id)
}

type CargoToml = HashMap<String, toml::Value>;

/// Parse the given `Cargo.toml` into a `HashMap`
fn parse_cargo_toml(file: &Path) -> CargoToml {
	let mut content = String::new();
	File::open(file)
		.expect("Cargo.toml exists")
		.read_to_string(&mut content)
		.expect("Reads file");
	toml::from_str(&content).expect("Cargo.toml is a valid toml file")
}

/// Replaces all substrate path dependencies with a git dependency.
fn replace_path_dependencies_with_git(
	cargo_toml_path: &Path,
	commit_id: &str,
	cargo_toml: &mut CargoToml,
) {
	let mut cargo_toml_path = cargo_toml_path.to_path_buf();
	// remove `Cargo.toml`
	cargo_toml_path.pop();

	for &table in &["dependencies", "build-dependencies", "dev-dependencies"] {
		let mut dependencies: toml::value::Table =
			match cargo_toml.remove(table).and_then(|v| v.try_into().ok()) {
				Some(deps) => deps,
				None => continue,
			};

		let deps_rewritten = dependencies
			.iter()
			.filter_map(|(k, v)| {
				v.clone().try_into::<toml::value::Table>().ok().map(move |v| (k, v))
			})
			.filter(|t| {
				t.1.contains_key("path") && {
					// if the path does not exists, we need to add this as git dependency
					t.1.get("path")
						.unwrap()
						.as_str()
						.map(|path| !cargo_toml_path.join(path).exists())
						.unwrap_or(false)
				}
			})
			.map(|(k, mut v)| {
				// remove `path` and add `git` and `rev`
				v.remove("path");
				v.insert("git".into(), PROJECT_GIT_URL.into());
				v.insert("rev".into(), commit_id.into());

				(k.clone(), v.into())
			})
			.collect::<HashMap<_, _>>();

		dependencies.extend(deps_rewritten.into_iter());

		cargo_toml.insert(table.into(), dependencies.into());
	}
}

/// Update the top level (workspace) `Cargo.toml` file.
///
/// - Adds `profile.release` = `panic = unwind`
/// - Adds `workspace` definition
fn update_top_level_cargo_toml(
	cargo_toml: &mut CargoToml,
	workspace_members: Vec<&PathBuf>,
	node_template_generated_folder: &Path,
) {
	let mut panic_unwind = toml::value::Table::new();
	panic_unwind.insert("panic".into(), "unwind".into());

	let mut profile = toml::value::Table::new();
	profile.insert("release".into(), panic_unwind.into());

	cargo_toml.insert("profile".into(), profile.into());

	let members = workspace_members
		.iter()
		.map(|p| {
			p.strip_prefix(node_template_generated_folder)
				.expect("Workspace member is a child of the node template path!")
				.parent()
				// We get the `Cargo.toml` paths as workspace members, but for the `members` field
				// we just need the path.
				.expect("The given path ends with `Cargo.toml` as file name!")
				.display()
				.to_string()
		})
		.collect::<Vec<_>>();

	let mut members_section = toml::value::Table::new();
	members_section.insert("members".into(), members.into());

	cargo_toml.insert("workspace".into(), members_section.into());
}

fn write_cargo_toml(path: &Path, cargo_toml: CargoToml) {
	let content = toml::to_string_pretty(&cargo_toml).expect("Creates `Cargo.toml`");
	let mut file = File::create(path).expect(&format!("Creates `{}`.", path.display()));
	
	//parse toml string into toml_edit library
	let mut toml_doc = content.parse::<Document>().expect("invalid doc");

	//convert all dependency dot tables to inline tables ( { path="foo" } )
	toml_doc
	.clone()
	.iter()
	// filter out everything that is not a dependency table
	.filter(|(k, _)| k.contains("dependencies"))
	.filter_map(|(k, v)| v.as_table().map(|t| (k, t)))
	.for_each(|(k, t)| {
		t.iter()
		.for_each(|v| {
			//save the table and convert it to an inline_table
			let table = toml_doc[k][v.0].clone().into_value().unwrap();
			//save table as inline table
			toml_doc[k][v.0] = toml_edit::value(table);
		})
	});

	write!(file, "{}", toml_doc.to_string()).expect("Writes `Cargo.toml`");
}

/// Build and test the generated node-template
fn check_and_test(path: &Path, cargo_tomls: &[PathBuf]) {
	// Build node
	assert!(Command::new("cargo")
		.args(&["check", "--all"])
		.current_dir(path)
		.status()
		.expect("Compiles node")
		.success());

	// Test node
	assert!(Command::new("cargo")
		.args(&["test", "--all"])
		.current_dir(path)
		.status()
		.expect("Tests node")
		.success());

	// Remove all `target` directories
	for toml in cargo_tomls {
		let mut target_path = toml.clone();
		target_path.pop();
		target_path = target_path.join("target");

		if target_path.exists() {
			fs::remove_dir_all(&target_path)
				.expect(&format!("Removes `{}`", target_path.display()));
		}
	}
}

fn main() {
    let args: Vec<String> = env::args().collect();

	if args.len() != 2{
		println!("Please specify a config file");
		return 
	}

    let config_file = &args[1];

    println!("Using config file {}", config_file);

	let contents = fs::read_to_string(config_file)
		.expect("Something went wrong reading the TOML config_file... missing file?");

	let config: Config = toml::from_str(&contents)
		.expect("Something went wrong reading the TOML config_file... not a TOML file?");

	copy_node_template(&config);

	let mut cargo_tomls = find_cargo_tomls(&config.output.path);

	let commit_id = get_git_commit_id(&config.upstream.source_path);
	let top_level_cargo_toml_path = &config.output.path.join("Cargo.toml");

	// Check if top level Cargo.toml exists. If not, create one in the destination
	if !cargo_tomls.contains(&top_level_cargo_toml_path) {
		// create the top_level_cargo_toml
		OpenOptions::new()
			.create(true)
			.write(true)
			.open(top_level_cargo_toml_path.clone())
			.expect("Create root level `Cargo.toml` failed.");

		// push into our data structure
		cargo_tomls.push(PathBuf::from(top_level_cargo_toml_path.clone()));
	}

	cargo_tomls.iter().for_each(|t| {
		let mut cargo_toml = parse_cargo_toml(&t);
		replace_path_dependencies_with_git(&t, &commit_id, &mut cargo_toml);

		// Check if this is the top level `Cargo.toml`, as this requires some special treatments.
		if top_level_cargo_toml_path == t {
			// All workspace member `Cargo.toml` file paths.
			let workspace_members =
				cargo_tomls.iter().filter(|p| *p != top_level_cargo_toml_path).collect();

			update_top_level_cargo_toml(&mut cargo_toml, workspace_members, &config.output.path);
		}

		write_cargo_toml(&t, cargo_toml);
	});

	// adding root rustfmt to node template build path
	let node_template_rustfmt_toml_path = &config.output.path.join("rustfmt.toml");
	let root_rustfmt_toml = &config.upstream.source_path.join("rustfmt.toml");
	if root_rustfmt_toml.exists() {
		fs::copy(&root_rustfmt_toml, &node_template_rustfmt_toml_path)
			.expect("Copying rustfmt.toml.");
	}

	if config.output.test {
		check_and_test(&config.output.path, &cargo_tomls);
	}
}
