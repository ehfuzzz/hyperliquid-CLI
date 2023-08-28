use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct OrderPayload {
    pub type_: String,
    pub grouping: String,
    pub orders: Vec<Orders>,
}
impl OrderPayload {
    pub fn new() -> OrderPayload {
        OrderPayload {
            type_: String::from("order"),
            grouping: String::from("na"),
            orders: Vec::new(),
        }
    }
    pub fn add_order(&mut self, order: Orders) {
        self.orders.push(order);
    }
}
#[derive(Serialize, Debug)]
pub struct Orders {
    pub asset: Option<u32>,
    pub isbuy: Option<bool>,
    pub limitpx: Option<String>,
    pub sz: Option<String>,
    pub reduceonly: Option<bool>,
    pub ordertype: Option<OrderType>,
}
#[derive(Serialize, Debug)]
pub enum OrderType {
    Limit(Limit),
    Trigger(Trigger),
}

impl Orders {
    pub fn new() -> Orders {
        Orders {
            asset: None,
            isbuy: None,
            limitpx: None,
            sz: None,
            reduceonly: None,
            ordertype: None,
        }
    }
    pub fn set_asset(&mut self, asset: u32) {
        self.asset = Some(asset);
    }
    pub fn get_asset(&self) -> u32 {
        self.asset.unwrap()
    }

    pub fn set_is_buy(&mut self, is_buy: bool) {
        self.isbuy = Some(is_buy);
    }

    pub fn set_limit_px(&mut self, limit_px: &str) {
        self.limitpx = Some(limit_px.to_string());
    }
    pub fn get_limit_px(&self) -> String {
        self.limitpx.clone().unwrap()
    }
    pub fn set_sz(&mut self, sz: &str) {
        self.sz = Some(sz.to_string());
    }
    pub fn get_sz(&self) -> String {
        self.sz.clone().unwrap()
    }
    pub fn set_reduce_only(&mut self, reduce_only: bool) {
        self.reduceonly = Some(reduce_only);
    }
    pub fn set_order_type(&mut self, order_type: OrderType) {
        self.ordertype = Some(order_type);
    }
}
#[derive(Serialize, Debug)]
pub struct Limit {
    pub tif: String,
}
impl Limit {
    pub fn new() -> Limit {
        Limit {
            tif: String::from("GTC"),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct Trigger {
    pub tpsl: String,
    pub trigger_px: Option<f64>,
    pub is_market: Option<bool>,
}

impl Trigger {
    pub fn new(trigger_type: &str) -> Trigger {
        Trigger {
            tpsl: String::from(trigger_type),
            trigger_px: None,
            is_market: None,
        }
    }

    pub fn set_trigger_px(&mut self, trigger_px: f64) {
        self.trigger_px = Some(trigger_px);
    }

    pub fn set_is_market(&mut self, is_market: bool) {
        self.is_market = Some(is_market);
    }
}
#[derive(Serialize, Debug)]
pub enum GainOptions {
    PercentageGain(f64),
    DollarGain(f64),
}

#[derive(Serialize, Debug)]
pub struct RequestBody {
    #[serde(rename = "type")]
    pub action: OrderPayload,
    pub nonce: u128,
    pub signature: String,
    pub vaultaddress: Option<String>,
}
