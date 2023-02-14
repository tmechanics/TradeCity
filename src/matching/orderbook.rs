use std::{collections::{VecDeque, HashMap}, fmt};

struct BuySellQueue {
    sell_orders: VecDeque<i32>,
    buy_orders: VecDeque<i32>
}

impl BuySellQueue {
    fn new() -> Self {
        BuySellQueue { 
            sell_orders: VecDeque::new(), 
            buy_orders: VecDeque::new() 
        }
    }
}

pub struct Orderbook<'a> {
    security: &'a Security,
    starting_price: i32,
    current_market_price: i32,
    best_bid: i32,  // the lowest price a buy order gets placed
    best_ask: i32,  // the hightest price a sell order gets placed
    worst_bid: i32, // the highest price a buy order gets placed
    worst_ask: i32, // the lowest price a sell order gets placed
    order_map: HashMap<i32, Order<'a>>,
    at_market_orders: VecDeque<i32>,
    limit_orders: VecDeque<BuySellQueue>,
}

impl <'a> Orderbook<'a> {
    fn new(security: &'a Security, starting_price: i32) -> Self {
        Orderbook {
            security,
            starting_price,
            current_market_price: starting_price,
            best_bid: 0,
            best_ask: 0,
            worst_bid: 0,
            worst_ask: 0,
            order_map: HashMap::new(),
            at_market_orders: VecDeque::new(),
            limit_orders: VecDeque::new(),
        }
    }

    fn place_order(&mut self, mut order: Order<'a>) -> Result<i32, OrderError> {
        if order.amount <= 0 { return Err(OrderError("Order amount must be greater than zero".to_string())); }

        let new_order_id = 1;
        order.order_id = new_order_id;
        let order_limit = order.order_limit;
        let is_buy_order = order.is_buy_order;
        self.order_map.insert(new_order_id, order);

        if let Some(limit) = order_limit {
            if limit <= 0 { return Err(OrderError("Limit must be greater than zero".to_string())); }

            if is_buy_order {
                if limit < self.best_bid {
                    let mut bsq = BuySellQueue::new();
                    bsq.buy_orders.push_back(new_order_id);
                    self.limit_orders.push_front(bsq);
                    self.best_bid = limit;
                    return Ok(new_order_id);
                }

                if limit > self.worst_bid {
                    let mut bsq = BuySellQueue::new();
                    bsq.buy_orders.push_back(new_order_id);
                    self.limit_orders.push_back(bsq);
                    self.worst_bid = limit;
                    return Ok(new_order_id);
                }

                let index = limit - self.best_bid;
                if let Some(bsq) = self.limit_orders.get_mut(index.try_into().unwrap()) { bsq.buy_orders.push_back(new_order_id); }
                return Ok(new_order_id);
            }

            if limit < self.best_ask {
                let mut bsq = BuySellQueue::new();
                bsq.sell_orders.push_back(new_order_id);
                self.limit_orders.push_front(bsq);
                self.best_ask = limit;
                return Ok(new_order_id);
            }

            if limit > self.worst_ask {
                let mut bsq = BuySellQueue::new();
                bsq.sell_orders.push_back(new_order_id);
                self.limit_orders.push_back(bsq);
                self.worst_ask = limit;
                return Ok(new_order_id);
            }

            let index = limit - self.best_ask;
            if let Some(bsq) = self.limit_orders.get_mut(index.try_into().unwrap()) { bsq.sell_orders.push_back(new_order_id); }
            return Ok(new_order_id);

        }

        self.at_market_orders.push_back(new_order_id);

        Ok(new_order_id)
    }

    fn delete_order(&mut self, order_id: i32) -> Result<(), OrderError> {
        match self.order_map.remove(&order_id) {
            Some(_) => Ok(()),
            None => Err(OrderError("No order found with given order_id".to_string())),
        }
    }

    fn execute(&mut self) {
        /* 
        It is inefficient to go through every order. Assuming, an order has been placed at price level something in the middle of the best and worst
        then it is useless to trigger the execution loop.

        Cases to trigger the execution loop
        ===================================
        1. New "at market" order is placed.
        2. A buy limit order with a new best bid is placed.
        3. A sell limit order with a new best ask is placed.
        */

    }


}
struct OrderError(String);

impl fmt::Display for OrderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

struct Order<'a> {
    order_id: i32,
    is_buy_order: bool,
    order_limit: Option<i32>,
    security: &'a Security,
    amount: i32,
}

struct Execution {

}

struct Security {
    isin: String,
    name: String,
}