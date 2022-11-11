use mysql_async::params;
use serde::{Deserialize, Serialize};

use super::common::{IpAddress, TimeStamp};
use super::sql::SqlTable;

#[derive(Debug, Deserialize, Serialize)]
pub struct TimerUnit {
    send_timestamp: TimeStamp,
    public_ip: IpAddress,
    category: String,
    tag: String,
    count: u64,
    max_time: u64,
    min_time: u64,
    avg_time: u64,
}

impl SqlTable for TimerUnit {
    type TypeSelf = TimerUnit;
    fn new_sql_table_opt() -> &'static str {
        r#"
        CREATE TABLE metrics_timer(
            send_timestamp INT(10) DEFAULT 0,
            public_ip VARCHAR(40) DEFAULT "",
            category VARCHAR(30) DEFAULT "",
            tag VARCHAR(100) DEFAULT "",
            count BIGINT(20) DEFAULT 0,
            max_time BIGINT(20) DEFAULT 0,
            min_time BIGINT(20) DEFAULT 0,
            avg_time BIGINT(20) DEFAULT 0,
            INDEX(category,tag,public_ip,send_timestamp)
        )ENGINE = InnoDB DEFAULT CHARSET = utf8;
        "#
    }

    fn insert_table_opt() -> &'static str {
        r#"
        INSERT INTO metrics_timer ( send_timestamp, public_ip, category, tag, count, max_time, min_time, avg_time )
        VALUES (:send_timestamp, :public_ip, :category, :tag, :count, :max_time, :min_time, :avg_time )
        "#
    }

    fn to_params(&self) -> mysql_async::Params {
        params! {
            "send_timestamp" => self.send_timestamp.data(),
            "public_ip" => self.public_ip.to_string(),
            "category" => self.category.clone(),
            "tag" => self.tag.clone(),
            "count" => self.count,
            "max_time" => self.max_time,
            "min_time" => self.min_time,
            "avg_time" => self.avg_time,
        }
    }
}
