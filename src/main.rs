use anyhow::{anyhow, Error};

use binance::api::*;
use binance::market::*;

use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;

enum NextOperation {
    Buy,
    Sell,
}

fn get_near_price() -> Result<Decimal, Error> {
    let market: Market = Binance::new(None, None);

    match market.get_price("NEARUSDT") {
        Ok(result) => Ok(Decimal::from_f64(result.price).unwrap()),
        Err(e) => Err(anyhow!(
            "Wasn't able to get current NEAR price due to: {:?}!",
            e
        )),
    }
}

fn buy(money: Decimal, near_price: Decimal) -> Result<Decimal, Error> {
    match near_price.is_zero() {
        true => Err(anyhow!("Can't buy NEAR for 0.00$!")),
        false => Ok(money / near_price),
    }
}

fn sell(near_amount: Decimal, near_price: Decimal) -> Result<Decimal, Error> {
    match near_price.is_zero() {
        true => Err(anyhow!("Can't sell NEAR for 0.00$!")),
        false => Ok(near_amount * near_price),
    }
}

fn main() {
    pretty_env_logger::init();

    let mut total_near_amount = Decimal::new(256, 0);
    let mut total_money_amount = Decimal::ZERO;

    let mut bought_for = match get_near_price() {
        Ok(price) => price,
        Err(e) => {
            log::error!("{}", e);

            let mut res: Result<Decimal, Error> = Err(anyhow!("Temp error"));
            while res.is_err() {
                res = get_near_price();
            }

            res.unwrap()
        }
    };
    let mut sold_for = Decimal::ZERO;

    let mut next_operation = NextOperation::Sell;
    loop {
        match next_operation {
            NextOperation::Buy => match get_near_price() {
                Ok(current_price) => {
                    println!("{}", sold_for - current_price);

                    if sold_for - current_price >= Decimal::new(2, 3) {
                        match buy(total_money_amount, current_price) {
                            Ok(near_amount) => {
                                total_near_amount += near_amount;
                                total_money_amount = Decimal::ZERO;

                                bought_for = current_price;
                                next_operation = NextOperation::Sell;

                                log::info!(
                                    "BOUGHT for: {}\nNEAR AMOUNT: {}\nMONEY AMOUNT: {}",
                                    current_price.to_string(),
                                    total_near_amount.to_string(),
                                    total_money_amount.to_string()
                                );
                            }
                            Err(e) => log::error!("{}", e),
                        }
                    }
                }
                Err(e) => log::error!("{}", e),
            },
            NextOperation::Sell => match get_near_price() {
                Ok(current_price) => {
                    println!("{}", current_price - bought_for);

                    if current_price - bought_for >= Decimal::new(2, 3) {
                        let five_percent =
                            total_near_amount / Decimal::new(100, 0) * Decimal::new(5, 0);
                        match sell(five_percent, current_price) {
                            Ok(money) => {
                                total_money_amount += money;
                                total_near_amount -= five_percent;

                                sold_for = current_price;
                                next_operation = NextOperation::Buy;

                                log::info!(
                                    "SOLD for: {}\nNEAR AMOUNT: {}\nMONEY AMOUNT: {}",
                                    current_price.to_string(),
                                    total_near_amount.to_string(),
                                    total_money_amount.to_string()
                                );
                            }
                            Err(e) => log::error!("{}", e),
                        }
                    }
                }
                Err(e) => log::error!("{}", e),
            },
        }

        std::thread::sleep(std::time::Duration::from_secs(15 * 60));
    }
}
