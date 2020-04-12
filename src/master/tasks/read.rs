use crate::app::format::write::{start_request, HeaderWriter};
use crate::app::gen::enums::FunctionCode;
use crate::app::header::{Control, ResponseHeader};
use crate::app::parse::parser::HeaderCollection;
use crate::app::sequence::Sequence;
use crate::master::handlers::ResponseHandler;
use crate::master::task::{ResponseError, ResponseResult};
use crate::master::types::ClassScan;
use crate::util::cursor::{WriteCursor, WriteError};

#[derive(Copy, Clone)]
pub enum ReadRequest {
    ClassScan(ClassScan),
}

impl ReadRequest {
    pub fn class_scan(scan: ClassScan) -> Self {
        ReadRequest::ClassScan(scan)
    }

    pub(crate) fn format(self, writer: &mut HeaderWriter) -> Result<(), WriteError> {
        match self {
            ReadRequest::ClassScan(scan) => scan.write(writer),
        }
    }
}

pub(crate) struct ReadTask {
    pub(crate) request: ReadRequest,
    pub(crate) handler: Box<dyn ResponseHandler>,
}

impl ReadTask {
    pub(crate) fn format(&self, seq: Sequence, cursor: &mut WriteCursor) -> Result<(), WriteError> {
        let mut writer = start_request(Control::request(seq), FunctionCode::Read, cursor)?;
        self.request.format(&mut writer)
    }

    pub(crate) fn handle(
        &mut self,
        response: ResponseHeader,
        headers: HeaderCollection,
    ) -> Result<ResponseResult, ResponseError> {
        // TODO - provide the proper addressing
        self.handler.handle(1024, response, headers);
        Ok(ResponseResult::Success)
    }
}