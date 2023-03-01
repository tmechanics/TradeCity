use std::collections::{VecDeque, HashMap};

pub struct Orderbook<'a> {
    security: &'a Security,
    starting_price: i64,
    current_market_price: i64,
    best_bid: i64,
    best_ask: i64,
    worst_bid: i64,
    worst_ask: i64,
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

    fn insert_order(&mut self, order: &mut Order<'a>) -> Result<(i64, MatchingSignal), String> {
        if order.amount <= 0 { return Err("Order amount must be greater than zero".to_string()); }

        let new_order_id: i64 = 1;
        order.order_id = new_order_id;
        let order_limit = order.order_limit;
        let is_buy_order = order.is_buy_order;
        self.buy_at_market_orders.len();

        if let Some(limit) = order_limit {
            if limit <= 0 { return Err("Limit must be greater than zero".to_string()); }

            if is_buy_order {
                if limit > self.best_bid {
                    let mut queue = VecDeque::new();
                    queue.push_back(new_order_id);
                    self.buy_limit_orders.push_front(queue);
                    self.number_buy_limit_orders += 1;
                    self.best_bid = limit;
                    return Ok((new_order_id, MatchingSignal::NewHighestBid),);
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
                
                return Ok((new_order_id, if index == 0 { MatchingSignal::NewHighestBid } else { MatchingSignal::NoOperation } ));
            }

            if limit < self.best_ask {
                let mut queue = VecDeque::new();
                queue.push_back(new_order_id);
                self.sell_limit_orders.push_front(queue);
                self.number_sell_limit_orders += 1;
                self.best_ask = limit;
                return Ok((new_order_id, MatchingSignal::NewLowestAsk));
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

            return Ok((new_order_id, if index == 0 { MatchingSignal::NewLowestAsk } else { MatchingSignal::NoOperation }));
        }

        Ok((new_order_id, if is_buy_order { MatchingSignal::BuyAtMarket } else { MatchingSignal::SellAtMarket }))
    }

    pub fn cancel_order(&mut self, order_id: i64) -> Result<(), String> {
        // The order needs to be removed from the order map as well as from the order queues.
        // With the help of the order metadata it should be relatively easy to find the order and remove it. 
        //
        // Other tasks: Decrement counter, new best bid, new worst bid, new best ask, new worst bid
        
        todo!()
        
    }

    pub fn place_order(&mut self, mut order: Order<'a>) -> Result<i64, String> {
        match self.insert_order(&mut order) {
            Ok((order_id, matching_signal)) => {
                match matching_signal {
                    MatchingSignal::BuyAtMarket => {
                        // try to match order directly
                        self.match_against_sell_side_at_market(&mut order);
                    },
                    MatchingSignal::SellAtMarket => {
                        // try to match order directly
                        self.match_against_buy_side_at_market(&mut order);
                    },
                    MatchingSignal::NewHighestBid => {
                        // try to match any orders on the sell side
                        self.match_against_sell_side(&mut order);
                    },
                    MatchingSignal::NewLowestAsk => {
                        // try to match any orders on the sell side
                        self.match_against_buy_side(&mut order);
                    },
                    MatchingSignal::NoOperation => {
                        self.order_map.insert(order_id, order);
                        // No further actions
                    },
                }
            
                return Ok(order_id)
            },
            Err(error_text) => Err(error_text),
        }
    }

    fn match_against_buy_side_at_market(&mut self, order: &mut Order<'a>) {
        // move to order map if no matching is possible or the order is not fully executed
        // first match with limit orders in order of the price and queue location on the buy side

        // match order and send to the accounting module
    }

    fn match_against_sell_side_at_market(&mut self, order: &mut Order<'a>) {
        // move to order map if no matching is possible or the order is not fully executed
        // first match with limit orders in order of the price and queue location on the sell side

        // match order and send to the accounting module
    }

    fn match_against_buy_side(&mut self, order: &mut Order<'a>) {
        // move to order map if no matching is possible or the order is not fully executed
        // first match with at markets order in order of the queue location on the buy side
        // then match with limit orders which are higher or equal than this one

        // match order and send to the accounting module
    }

    fn match_against_sell_side(&mut self, order: &mut Order<'a>) {
        // move to order map if no matching is possible or the order is not fully executed
        // first match with market order in order of the queue location on the sell side
        // then match with limit orders which are lower or equal price than this one

        // match order and send to the accounting module 
    }
}

enum MatchingSignal {
    BuyAtMarket,
    SellAtMarket,
    NewHighestBid,
    NewLowestAsk,
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