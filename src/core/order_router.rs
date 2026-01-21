use crate::models::{OrderAck, OrderRequest};
use crate::Result;

pub trait OrderRouter {
    fn route(&self, req: OrderRequest) -> Result<OrderAck>;
}
