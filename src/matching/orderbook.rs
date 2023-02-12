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

    fn add_order(mut self, order_id: i32, is_buy_order: bool) {
        if is_buy_order { self.buy_orders.push_back(order_id) }
        else { self.sell_orders.push_back(order_id) }
    }
}

struct Orderbook<'a> {
    security: &'a Security,
    current_market_price: i32,
    best_bid: i32, 
    best_ask: i32,
    worst_bid: i32,
    worst_ask: i32,
    order_map: HashMap<i32, Order<'a>>,
    at_market_orders: BuySellQueue,
    limit_orders: VecDeque<BuySellQueue>,
}

impl <'a> Orderbook<'a> {
    fn new(security: &'a Security, ) -> Self {
        Orderbook {
            security,
            current_market_price: 0,
            best_bid: 0,
            best_ask: 0,
            worst_bid: 0,
            worst_ask: 0,
            order_map: HashMap::new(),
            at_market_orders: BuySellQueue::new(),
            limit_orders: VecDeque::new(),
        }
    }

    fn place_order(&mut self, mut order: Order<'a>) -> Result<i32, OrderError> {
        // TODO: Additional check, so that the limit is not to far away from the current level

        let new_order_id = 1;
        order.order_id = new_order_id;
        let order_limit = order.order_limit;
        let is_buy_order = order.is_buy_order;
        self.order_map.insert(new_order_id, order);

        if let Some(limit) = order_limit {
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

        if is_buy_order { self.at_market_orders.buy_orders.push_back(new_order_id); }
        else { self.at_market_orders.sell_orders.push_back(new_order_id); }

        Ok(new_order_id)
    }

    fn delete_order(&mut self, order_id: i32) -> Result<(), OrderError> {
        match self.order_map.remove(&order_id) {
            Some(_) => Ok(()),
            None => Err(OrderError("No order found with given order_id".to_string())),
        }
    }

    fn execute(&mut self) {
        // event based execution, so on every placement, one execution round has to be done

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