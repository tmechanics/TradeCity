use std::collections::{VecDeque, HashMap};

pub struct Orderbook<'a> {
    security: &'a Security,
    starting_price: i64,
    current_market_price: i64,
    best_bid: i64,  // the lowest price a buy order gets placed
    best_ask: i64,  // the hightest price a sell order gets placed
    worst_bid: i64, // the highest price a buy order gets placed
    worst_ask: i64, // the lowest price a sell order gets placed
    order_map: HashMap<i64, Order<'a>>,
    buy_at_market_orders: VecDeque<i64>,
    sell_at_market_orders: VecDeque<i64>,
    buy_limit_orders: VecDeque<VecDeque<i64>>,
    sell_limit_orders: VecDeque<VecDeque<i64>>,
    number_buy_limit_orders: u32,
    number_sell_limit_orders: u32,
}

impl <'a> Orderbook<'a> {
    pub fn new(security: &'a Security, starting_price: i64) -> Self {
        Orderbook {
            security,
            starting_price,
            current_market_price: starting_price,
            best_bid: 0,
            best_ask: 0,
            worst_bid: 0,
            worst_ask: 0,
            order_map: HashMap::new(),
            buy_at_market_orders: VecDeque::new(),
            sell_at_market_orders: VecDeque::new(),
            buy_limit_orders: VecDeque::new(),
            sell_limit_orders: VecDeque::new(),
            number_buy_limit_orders: 0,
            number_sell_limit_orders: 0
        }
    }

    fn insert_order(&mut self, mut order: Order<'a>) -> Result<(i64, MatchingSignal), String> {
        if order.amount <= 0 { return Err("Order amount must be greater than zero".to_string()); }

        let new_order_id: i64 = 1;
        order.order_id = new_order_id;
        let order_limit = order.order_limit;
        let is_buy_order = order.is_buy_order;
        self.order_map.insert(new_order_id, order);

        if let Some(limit) = order_limit {
            if limit <= 0 { return Err("Limit must be greater than zero".to_string()); }

            if is_buy_order {
                if limit > self.best_bid {
                    let mut queue = VecDeque::new();
                    queue.push_back(new_order_id);
                    self.buy_limit_orders.push_front(queue);
                    self.number_buy_limit_orders += 1;
                    self.best_bid = limit;
                    return Ok((new_order_id, MatchingSignal::NewLowestBid));
                }

                if limit < self.worst_bid {
                    if self.worst_bid * 12 < self.current_market_price * 10 { return Err("Limit is too far away from current market price.".to_string()); }
                    let mut queue = VecDeque::new();
                    queue.push_back(new_order_id);
                    self.buy_limit_orders.push_back(queue);
                    self.number_buy_limit_orders += 1;
                    self.worst_bid = limit;
                    return Ok((new_order_id, MatchingSignal::NoOperation));
                }

                let index = limit - self.best_bid;
                if let Some(subqueue) = self.buy_limit_orders.get_mut(index.try_into().unwrap()) {
                    subqueue.push_back(new_order_id);    
                } else {
                    let mut queue = VecDeque::new();
                    queue.push_back(new_order_id);
                    self.buy_limit_orders.insert(index.try_into().unwrap(), queue);
                }
                
                return Ok((new_order_id, MatchingSignal::NoOperation));
            }

            if limit < self.best_ask {
                let mut queue = VecDeque::new();
                queue.push_back(new_order_id);
                self.sell_limit_orders.push_front(queue);
                self.number_sell_limit_orders += 1;
                self.best_ask = limit;
                return Ok((new_order_id, MatchingSignal::NewHighestAsk));
            }

            if limit > self.worst_ask {
                let mut queue = VecDeque::new();
                queue.push_back(new_order_id);
                self.sell_limit_orders.push_back(queue);
                self.number_sell_limit_orders += 1;
                self.worst_ask = limit;
                return Ok((new_order_id, MatchingSignal::NoOperation));
            }

            let index = limit - self.best_ask;
            if let Some(subqueue) = self.sell_limit_orders.get_mut(index.try_into().unwrap()) {
                subqueue.push_back(new_order_id);
            } else {
                let mut queue = VecDeque::new();
                queue.push_back(new_order_id);
                self.sell_limit_orders.insert(index.try_into().unwrap(), queue);
            }

            return Ok((new_order_id, MatchingSignal::NoOperation));
        }

        if is_buy_order {
            self.buy_at_market_orders.push_back(new_order_id);
            Ok((new_order_id, MatchingSignal::BuyAtMarket))
        } else {
            self.sell_at_market_orders.push_back(new_order_id);
            Ok((new_order_id, MatchingSignal::SellAtMarket))
        }
    }

    pub fn cancel_order(&mut self, order_id: i64) -> Result<(), String> {
        // The order needs to be removed from the order map as well as from the order queues.
        // With the help of the order metadata it should be relatively easy to find the order and remove it. 
        //
        // Other tasks: Decrement counter, new best bid, new worst bid, new best ask, new worst bid
        
        todo!()
        
    }

    pub fn place_order(&mut self, order: Order<'a>) -> Result<i64, String> {
        match self.insert_order(order) {
            Ok((order_id, MatchingSignal::NoOperation)) => Ok(order_id),
            Ok((order_id, matching_signal)) => {
                self.start_execution_round(matching_signal);
                return Ok(order_id)
            },
            Err(error_text) => Err(error_text),
        }
    }

    fn start_execution_round(&mut self, matching_signal: MatchingSignal) -> Result<Vec<Execution>, String> {
        match matching_signal {
            MatchingSignal::BuyAtMarket => Ok(vec![]),  
            MatchingSignal::NewLowestBid => Ok(vec![]), 
            MatchingSignal::SellAtMarket => Ok(vec![]),
            MatchingSignal::NewHighestAsk => Ok(vec![]),
            MatchingSignal::NoOperation => Ok(vec![]),
        }
    }
}

enum MatchingSignal {
    BuyAtMarket,
    NewLowestBid,
    SellAtMarket,
    NewHighestAsk,
    NoOperation
}

pub struct Order<'a> {
    order_id: i64,
    is_buy_order: bool,
    order_limit: Option<i64>,
    security: &'a Security,
    amount: i64,
    amount_executed: i64
}

impl <'a> Order<'a> {
    pub fn new(is_buy_order: bool, order_limit: Option<i64>, security: &Security, amount: i64) -> Order {
        Order {
            order_id: -1,
            is_buy_order,
            order_limit,
            security,
            amount,
            amount_executed: 0,
        }
    }

    pub fn order_id(&self) -> i64 {
        self.order_id
    }
}

struct Execution<'a> {
    selling_order: &'a Order<'a>,
    buying_order: &'a Order<'a>,
    amount: i64,
}

pub struct Security {
    pub isin: String,
    pub name: String,
}