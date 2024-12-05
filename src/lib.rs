mod parse;
mod tokenize;
mod value;

use crate::parse::{parse_tokens, TokenParseError};
use crate::tokenize::{tokenize, TokenizeError};
use crate::value::Value;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    TokenizeError(TokenizeError),
    TokenParseError(TokenParseError),
}

impl From<TokenParseError> for ParseError {
    fn from(e: TokenParseError) -> Self {
        Self::TokenParseError(e)
    }
}

impl From<TokenizeError> for ParseError {
    fn from(e: TokenizeError) -> Self {
        Self::TokenizeError(e)
    }
}

pub fn parse(input: &str) -> Result<Value, ParseError> {
    let tokens = tokenize(input)?;
    let value = parse_tokens(&tokens, &mut 0)?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_test() {
        let input = r#"
        {"basic_info":{"title":"瓜分奖","time_text":"04月-20日 02:00-05月-29日 12:00","start_time":"04.20 02:00:00","activity_id":2199039482869,"attention_text":"权益说明","enroll_type":1,"enroll_info":{"status":" need_enroll"},"activity_type":"terra_divide_reward","order_types":[201,202,203,204,205,208,210,220],"driver_id":580542143947406,"product_level":[260],"origin_status":"not_start","user_city":0,"head_tip":"完成30单\\n参与瓜分{19.2万元}","status":"not_start"},"reward_task":{"restrict_info":{"progress_pancel_v3":{"desc_text":"保持排名，结束后可瓜分{11.2万元}"},"text":"保持排名，结束后可瓜分{11.2万元}","order_type":["快车单","特惠快车单","滴滴特快单","优享单","拼车单","特惠快车抢单模式","特惠快车单（仅轻快司机）","自选车"],"order_type_text":"不包含拼车一口价单","strive_type":["实时单","预约指派订单","预约单抢单接单"],"region_type":["顺路目的地","顺路区域","非顺路订单"],"city_list":["北京市（仅限东城区、西城区）"],"threshold":{"cal_rule":"30%单量+40%流水","rank_rule":[{"percent":"前10%","divide_amount":"{1}万元"},{"percent":"前10-40%","divide_amount":"{2.0001}万元"},{"percent":"前40-100%","divide_amount":"{3}万元"}]},"activity_rank_info":[{"cur_list":[{"stage":1,"rank":1,"score":100.25,"user_id":1,"order_cnt":100,"order_income":10000.01},{"stage":1,"rank":10,"score":90.25,"user_id":10,"order_cnt":50,"order_income":5000.01}],"percent":"排名前10%"},{"cur_list":[{"stage":2,"rank":11,"score":80.25,"user_id":11,"order_cnt":20,"order_income":500.01},{"stage":2,"rank":20,"score":70.25,"user_id":20,"order_cnt":20,"order_income":200.91}],"percent":"排名前10-40%"},{"cur_list":[{"stage":3,"rank":21,"score":50.25,"user_id":21,"order_cnt":10,"order_income":120.01},{"stage":3,"rank":50,"score":30.25,"user_id":50,"order_cnt":1,"order_income":10.91}],"percent":"排名前40-100%"}],"my_rank_info":{"order_cnt":7,"total_income":70,"score":30.1,"rank":3,"cur_stage":1,"cur_stage_amount":10000},"geo_info":{"start":[{"id":"3908367","desc":"起点范围"}],"end":[{"id":"3908367","desc":"起点范围"}]}}}}
        "#;
        let parsed = parse(input).unwrap();
        println!("{:?}", parsed);
    }
}
