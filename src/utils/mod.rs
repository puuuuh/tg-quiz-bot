use telegram_bot::{Request, Api, ResponseType, UpdateKind, Message, MessageChat, ChatId, ChannelPost};
use tokio::time::Duration;

pub async fn must_send<Req: Request>(api: &Api, req: Req) -> Option<<Req::Response as ResponseType>::Type> {
    loop {
        match api.send(&req).await {
            Ok(e) => {
                return Some(e)
            }
            Err(e) => {
                println!("{}", e);   
                if !e.to_string().contains("Too Many Requests") {
                    return None 
                } else {
                    tokio::time::delay_for(Duration::from_secs(5)).await;
                    continue
                }
            }
        }
    }
}

