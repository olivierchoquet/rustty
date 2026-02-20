use flexi_logger::writers::LogWriter;

pub struct DashboardWriter {
    sender: tokio::sync::mpsc::UnboundedSender<String>,
}

impl DashboardWriter {
    pub fn new(sender: tokio::sync::mpsc::UnboundedSender<String>) -> Self {
        Self { sender }
    }
}

impl LogWriter for DashboardWriter {
    fn write(&self, _now: &mut flexi_logger::DeferredNow, record: &log::Record) -> std::io::Result<()> {
        let msg = format!("{}", record.args());
        
        // C'est ce print qu'on veut voir !
        println!("LOGWRITER EN ACTION : {}", msg); 
        
        let _ = self.sender.send(msg);
        Ok(())
    }

    fn flush(&self) -> std::io::Result<()> {
        Ok(())
    }
}