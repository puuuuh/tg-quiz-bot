use telegram_bot::{Request, Api, ResponseType, UpdateKind, Message, MessageChat, ChatId, ChannelPost};
use tokio::time::Duration;

pub async fn must_send<Req: Request>(api: &Api, req: Req) -> <Req::Response as ResponseType>::Type {
    loop {
        match api.send(&req).await {
            Ok(e) => {
                return e
            }
            Err(e) => {
                println!("{}", e);
                tokio::time::delay_for(Duration::from_secs(5)).await;
                continue
            }
        }
    }
}

