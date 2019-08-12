use actix::prelude::*;

use witnet_net::client::tcp::jsonrpc;

use crate::actors::app;

impl Handler<jsonrpc::Notification> for app::App {
    type Result = <jsonrpc::Notification as Message>::Result;

    fn handle(
        &mut self,
        jsonrpc::Notification(value): jsonrpc::Notification,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.handle_block_notification(value)
    }
}