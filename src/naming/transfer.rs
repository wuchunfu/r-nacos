use crate::naming::core::NamingActor;
use crate::transfer::model::{TransferDataRequest, TransferDataResponse};
use actix::Handler;

impl Handler<TransferDataRequest> for NamingActor {
    type Result = anyhow::Result<TransferDataResponse>;

    fn handle(&mut self, msg: TransferDataRequest, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            TransferDataRequest::Backup(writer_actor, param) => {
                if param.naming {
                    self.transfer_backup(writer_actor)?;
                }
                Ok(TransferDataResponse::None)
            }
        }
    }
}
