use crate::types::{self, Order};
use anyhow::Result;
use log::info;
use nostr::util::time::timestamp;
use nostr::{EventBuilder, Kind};
use nostr_sdk::nostr::Keys;
use nostr_sdk::Client;
use sqlx::SqlitePool;

pub async fn publish_order(
    pool: &SqlitePool,
    client: &Client,
    keys: &Keys,
    order: &Order,
    initiator_pubkey: &str,
) -> Result<()> {
    let order = Order::new(
        order.kind,
        types::Status::Pending,
        order.amount,
        order.fiat_code.to_owned(),
        order.fiat_amount,
        order.payment_method.to_owned(),
        order.prime,
        None,
        Some(timestamp()),
    );
    let order_string = order.as_json().unwrap();
    let event = EventBuilder::new(Kind::Custom(11000), &order_string, &[])
        .to_event(keys)
        .unwrap();
    let event_id = event.id.to_string();

    info!("Event published: {:#?}", event);
    let order_id = crate::db::add_order(pool, &order, &event_id, initiator_pubkey).await?;
    info!("New order saved Id: {order_id}");

    client
        .send_event(event)
        .await
        .map(|_s| ())
        .map_err(|err| err.into())
}
