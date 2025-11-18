mod has_output;
mod in_process;
mod pre;
mod ready_to_draw;

pub(super) use has_output::HasOutput;
pub(super) use in_process::InProcess;
pub(super) use pre::Pre;
pub(super) use ready_to_draw::ReadyToDraw;

//pub(crate) use self::help_template::HelpTemplate;

#[derive(Debug, Clone)]
pub enum BufferSurface {
    Pre(Pre),
    InProcess(InProcess),
    HasOutput(HasOutput),
    ReadyToDraw(ReadyToDraw),
}
