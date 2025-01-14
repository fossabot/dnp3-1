use std::sync::{Arc, Mutex};

use crate::app::Timestamp;
use crate::outstation::database::DatabaseHandle;
use crate::outstation::tests::harness::{Event, EventSender};
use crate::outstation::traits::{OutstationApplication, RequestError, RestartDelay};
use crate::outstation::{FreezeIndices, FreezeType};

pub(crate) struct MockOutstationApplication {
    events: EventSender,
    data: Arc<Mutex<ApplicationData>>,
}

pub(crate) struct ApplicationData {
    pub(crate) processing_delay: u16,
    pub(crate) restart_delay: Option<RestartDelay>,
}

impl ApplicationData {
    fn new() -> Self {
        Self {
            processing_delay: 0,
            restart_delay: None,
        }
    }
}

impl MockOutstationApplication {
    pub(crate) fn new(
        events: EventSender,
    ) -> (Arc<Mutex<ApplicationData>>, Box<dyn OutstationApplication>) {
        let data = Arc::new(Mutex::new(ApplicationData::new()));
        (data.clone(), Box::new(Self { events, data }))
    }
}

impl OutstationApplication for MockOutstationApplication {
    fn write_absolute_time(&mut self, time: Timestamp) -> Result<(), RequestError> {
        self.events.send(Event::WriteAbsoluteTime(time));
        Ok(())
    }

    fn get_processing_delay_ms(&self) -> u16 {
        self.data.lock().unwrap().processing_delay
    }

    fn cold_restart(&mut self) -> Option<RestartDelay> {
        let delay = self.data.lock().unwrap().restart_delay;
        self.events.send(Event::ColdRestart(delay));
        delay
    }

    fn warm_restart(&mut self) -> Option<RestartDelay> {
        let delay = self.data.lock().unwrap().restart_delay;
        self.events.send(Event::WarmRestart(delay));
        delay
    }

    fn freeze_counter(
        &mut self,
        indices: FreezeIndices,
        freeze_type: FreezeType,
        _db: &mut DatabaseHandle,
    ) -> Result<(), RequestError> {
        self.events.send(Event::Freeze(indices, freeze_type));
        Ok(())
    }
}
