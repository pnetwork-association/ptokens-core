mod cli_args;
mod write_file;

pub(crate) mod get_latest_block_num;
pub(crate) mod get_sub_mat;

pub(crate) use self::{
    cli_args::{CliArgs, SubCommands},
    write_file::write_file,
};
