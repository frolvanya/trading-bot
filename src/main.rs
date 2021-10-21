use anyhow::{anyhow, Error};

use binance::api::*;
use binance::market::*;

enum NextOperation {
    Buy,
    Sell,
}

fn get_near_price() -> Result<f64, Error> {
    let market: Market = Binance::new(None, None);

    match market.get_price("NEARUSDT") {
        Ok(result) => Ok(result.price),
        Err(e) => Err(anyhow!(
            "Wasn't able to get current NEAR price due to: {:?}!",
            e
        )),
    }
}

fn buy(money: f64, near_price: f64) -> Result<f64, Error> {
    match (near_price * 10_f64.powf(10_f64)).trunc() as u128 {
        0 => Err(anyhow!("Can't buy NEAR for 0.00$!")),
        _ => Ok(money / near_price),
    }
}

fn sell(near_amount: f64, near_price: f64) -> Result<f64, Error> {
    match (near_price * 10_f64.powf(10_f64)).trunc() as u128 {
        0 => Err(anyhow!("Can't sell NEAR for 0.00$!")),
        _ => Ok(near_amount * near_price),
    }
}

fn main() {
    pretty_env_logger::init();

    let mut total_near_amount: f64 = 256_f64;
    let mut total_money_amount: f64 = 0_f64;

    let mut bought_for: f64 = match get_near_price() {
        Ok(price) => price,
        Err(e) => {
            log::error!("{}", e);

            let mut res: Result<f64, Error> = Err(anyhow!("Temp error"));
            while res.is_err() {
                res = get_near_price();
            }

            res.unwrap()
        }
    };
    let mut sold_for: f64 = 0_f64;

    let mut next_operation = NextOperation::Sell;
    loop {
        match next_operation {
            NextOperation::Buy => {
                match get_near_price() {
                    Ok(current_price) => {
                        if sold_for - current_price >= 0.002 {
                            match buy(total_money_amount, current_price) {
                                Ok(near_amount) => {
                                    total_near_amount += near_amount;
                                    total_money_amount = 0_f64;

                                    bought_for = current_price;
                                    next_operation = NextOperation::Sell;

                                    log::info!("BOUGHT for: {:.02}\nNEAR AMOUNT: {:.02}\nMONEY AMOUNT: {:.02}", current_price, total_near_amount, total_money_amount);
                                }
                                Err(e) => log::error!("{}", e),
                            }
                        }
                    }
                    Err(e) => log::error!("{}", e),
                }
            }
            NextOperation::Sell => {
                match get_near_price() {
                    Ok(current_price) => {
                        if current_price - bought_for >= 0.002 {
                            let five_percent = total_near_amount / 100_f64 * 5_f64;
                            match sell(five_percent, current_price) {
                                Ok(money) => {
                                    total_money_amount += money;
                                    total_near_amount -= five_percent;

                                    sold_for = current_price;
                                    next_operation = NextOperation::Buy;

                                    log::info!("SOLD for: {:.02}\nNEAR AMOUNT: {:.02}\nMONEY AMOUNT: {:.02}", current_price, total_near_amount, total_money_amount);
                                }
                                Err(e) => log::error!("{}", e),
                            }
                        }
                    }
                    Err(e) => log::error!("{}", e),
                }
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
