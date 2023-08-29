use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct PlaceResponse {
    pub status: String,
    pub response: OrderResponse,
}

#[derive(Deserialize, Debug)]
pub struct OrderResponse {
    pub type_: String,
    pub data: OrderResponseData,
}

#[derive(Deserialize, Debug)]
pub struct OrderResponseData {
    pub statuses: Vec<OrderStatus>,
}

#[derive(Deserialize, Debug)]
pub struct OrderStatus {
    pub status: StatusOption,
}

#[derive(Deserialize, Debug)]
pub enum StatusOption {
    Resting(StatusType),
    Filled(StatusType),
}

#[derive(Deserialize, Debug)]
pub struct StatusType {
    pub oid: u64,
}
