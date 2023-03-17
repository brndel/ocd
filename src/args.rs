use clap::Parser;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct OcdArgs {
  /// The path to the folder of the project
  ///
  /// Example: "~/IdeaProjects/myProject"
  pub folder_path: String,
  /// The java path to the main class
  ///
  /// Example: "com.example.Main"
  pub main_class: String,
  /// The path to the folder containing the interaction files
  ///
  /// Example: "~/IdeaProjects/myProject/interactions"
  pub interactions_path: String,
  /// A regex pattern for file names to be considered interaction files
  ///
  /// [default: ".*\.txt"]
  ///
  /// [example: "interaction\.txt"]
  #[arg(short, long)]
  pub pattern: Option<String>,
  /// Whether to use multithreading while running the interactions
  ///
  /// [default: true]
  #[arg(short, long)]
  pub threads: Option<bool>,
}
