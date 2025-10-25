use std::path::PathBuf;

use fog_common::strum;
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

// impl TryFrom<String> for CliCommand
// {
//     type Error = anyhow::Error;

//     fn try_from(value: String) -> Result<CliCommand, Self::Error>
//     {
//         match value.as_str() {
//             "c" | "compile" => Ok(Self::Compile),
//             "r" | "run" => Ok(Self::Run),
//             "h" | "help" => Ok(Self::Help),
//             "v" | "version" => Ok(Self::Version),
//             "n" | "new" => Ok(Self::New),
//             "i" | "init" => Ok(Self::Init),
//             "l" | "link" => Ok(Self::Link),

//             _ => {
//                 println!("Invalid Argument: `{value}`");
//                 Err(CliParseError::InvalidArg(value).into())
//             },
//         }
//     }
// }
