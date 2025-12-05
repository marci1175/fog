use std::path::PathBuf;

use common::strum;
use strum::{Display, VariantNames};

#[derive(Clone, Debug, Display, clap::Subcommand, PartialEq, Eq, VariantNames)]
pub enum CliCommand
{
    /// Displays the compiler's version number.
    Version,
    /// Links an executable from a `.manifest` file.
    Link
    {
        #[arg(short, long, value_name = "MANIFEST_PATH")]
        path: PathBuf,
    },
    /// Publishes a dependency to the target url
    Publish
    {
        #[arg(
            short,
            long,
            value_name = "REMOTE_URL",
            help = "The address of the remote server, we are sending this dependency request to."
        )]
        url: String,

        #[arg(
            short,
            long,
            value_name = "AUTHOR_NAME",
            help = "The name the dependency should be published to (The name must match on following updates)."
        )]
        author: String,

        #[arg(
            short,
            long,
            value_name = "DEPENDENCY_SECRET",
            help = "Secret key to be able to update and modify a dependency."
        )]
        secret: Option<String>,

        #[arg(
            short,
            long,
            value_name = "PROJECT_PATH",
            help = "The path to the project we want published."
        )]
        path: Option<PathBuf>,
    },
    /// Compiles a project without running it.
    Compile
    {
        #[arg(short, long, default_value = None, help = "The path to the project's root. Default path is the current directory path.", value_name = "PROJECT_ROOT")]
        path: Option<PathBuf>,

        #[arg(
            short,
            long,
            default_value_t = false,
            help = "Whether the compiler output should be optimized or not."
        )]
        release: bool,

        #[arg(
            short,
            long,
            hide_default_value = true,
            help = "Specifies the compiler target. This must be a target triple supported by clang."
        )]
        target_triple: Option<String>,

        #[arg(
            short,
            long,
            default_value = "",
            hide_default_value = true,
            help = "Additional flags which can be passed in to llvm."
        )]
        llvm_flags: String,

        #[arg(
            short = 'n',
            long,
            help = "Sets the default CPU name of the LLVM target. If the argument is ignored, host values apply."
        )]
        cpu_name: Option<String>,

        #[arg(
            short = 'f',
            long,
            help = "Sets the default CPU features of the LLVM target. If the argument is ignored, host values apply."
        )]
        cpu_features: Option<String>,
    },
    /// Compiles a project and automatically runs it.
    Run
    {
        #[arg(short, long, default_value = None, help = "The path to the project's root. Default path is the current directory path.", value_name = "PROJECT_ROOT")]
        path: Option<PathBuf>,

        #[arg(
            short,
            long,
            default_value_t = false,
            help = "Whether the compiler output should be optimized or not."
        )]
        release: bool,

        #[arg(
            short,
            long,
            hide_default_value = true,
            help = "Specifies the compiler target. This must be a target triple supported by clang."
        )]
        target_triple: Option<String>,

        #[arg(
            short,
            long,
            default_value = "",
            hide_default_value = true,
            help = "Additional flags which can be passed in to llvm."
        )]
        llvm_flags: String,

        #[arg(
            short = 'n',
            long,
            help = "Sets the target CPU name of the LLVM target. If the argument is ignored, host values apply."
        )]
        cpu_name: Option<String>,

        #[arg(
            short = 'f',
            long,
            help = "Sets the default CPU features of the LLVM target. If the argument is ignored, host values apply."
        )]
        cpu_features: Option<String>,
    },
    /// Initializes a project.
    Init
    {
        #[arg(short, long, value_name = "FOLDER_DESTINATION")]
        path: PathBuf,
    },
    /// Creates a new folder with a new project inside it.
    New
    {
        #[arg(short, long, value_name = "NEW_PROJECT_PATH")]
        path: PathBuf,
    },
}
