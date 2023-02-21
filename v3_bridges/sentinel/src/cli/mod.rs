mod cli_args;
pub(crate) mod get_sub_mat;
mod write_file;

pub(crate) use self::{
    cli_args::CliArgs,
    get_sub_mat::{get_host_sub_mat, get_native_sub_mat},
    write_file::write_file,
};
