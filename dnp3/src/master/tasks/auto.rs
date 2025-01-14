use crate::app::format::write::HeaderWriter;
use crate::app::parse::parser::HeaderCollection;
use crate::app::FunctionCode;
use crate::app::ResponseHeader;
use crate::master::association::Association;
use crate::master::error::TaskError;
use crate::master::request::EventClasses;
use crate::master::tasks::{NonReadTask, Task};

use scursor::WriteError;

#[derive(Clone)]
pub(crate) enum AutoTask {
    ClearRestartBit,
    EnableUnsolicited(EventClasses),
    DisableUnsolicited(EventClasses),
}

impl AutoTask {
    pub(crate) fn wrap(self) -> Task {
        Task::NonRead(NonReadTask::Auto(self))
    }

    pub(crate) fn write(&self, writer: &mut HeaderWriter) -> Result<(), WriteError> {
        match self {
            AutoTask::ClearRestartBit => writer.write_clear_restart(),
            AutoTask::EnableUnsolicited(classes) => classes.write(writer),
            AutoTask::DisableUnsolicited(classes) => classes.write(writer),
        }
    }

    pub(crate) fn function(&self) -> FunctionCode {
        match self {
            AutoTask::ClearRestartBit => FunctionCode::Write,
            AutoTask::EnableUnsolicited(_) => FunctionCode::EnableUnsolicited,
            AutoTask::DisableUnsolicited(_) => FunctionCode::DisableUnsolicited,
        }
    }

    pub(crate) fn description(&self) -> &'static str {
        match self {
            AutoTask::ClearRestartBit => "clear restart IIN bit",
            AutoTask::EnableUnsolicited(_) => "enable unsolicited reporting",
            AutoTask::DisableUnsolicited(_) => "disable unsolicited reporting",
        }
    }

    pub(crate) fn handle(
        self,
        association: &mut Association,
        header: ResponseHeader,
        objects: HeaderCollection,
    ) -> Option<NonReadTask> {
        if !objects.is_empty() {
            tracing::warn!("ignoring object headers in reply to {}", self.description());
        }

        match &self {
            AutoTask::DisableUnsolicited(_) => {
                association.on_disable_unsolicited_response(header.iin);
            }
            AutoTask::EnableUnsolicited(_) => {
                association.on_enable_unsolicited_response(header.iin);
            }
            AutoTask::ClearRestartBit => {
                association.on_clear_restart_iin_response(header.iin);
            }
        };

        None
    }

    pub(crate) fn on_task_error(self, association: Option<&mut Association>, _err: TaskError) {
        if let Some(association) = association {
            match &self {
                AutoTask::DisableUnsolicited(_) => {
                    association.on_disable_unsolicited_failure();
                }
                AutoTask::EnableUnsolicited(_) => {
                    association.on_enable_unsolicited_failure();
                }
                AutoTask::ClearRestartBit => {
                    association.on_clear_restart_iin_failure();
                }
            }
        }
    }
}
