use mysql_async::Params;
use mysql_async::Result;

use mysql_async::prelude::*;
use mysql_async::{
    prelude::{Query, WithParams},
    Pool,
};

use crate::metrics_types::{sql::SqlTable, CounterUnit, FlowUnit, TimerUnit};

#[derive(Debug)]
pub struct MysqlDBConn {
    db_name: String,
    pool: Pool,
}

impl MysqlDBConn {
    // mysql_url : dw-consumer:consumerPswd!1@localhost:3306
    pub async fn new(mysql_url: String, db_name: String) -> Result<Self> {
        let default_opt = format!("mysql://{}/information_schema", &mysql_url);
        let target_opt = format!("mysql://{}/{}", &mysql_url, &db_name);

        let pool = mysql_async::Pool::new(default_opt.as_ref());

        // create db if not exist:
        let mut conn = pool.get_conn().await?;
        let sql = format!(r#"SHOW DATABASES LIKE "{}";"#, db_name);
        let db_exist_result = sql
            .map::<String, Option<String>, _, &mut mysql_async::Conn>(&mut conn, |db_result| {
                Some(db_result)
            })
            .await?;
        println!("db_exist_result:{:?}", db_exist_result);

        let mut need_create = false;

        if db_exist_result.is_empty() {
            let _ = format!(r#"CREATE DATABASE {};"#, db_name)
                .run(&mut conn)
                .await?;
            println!("CREATE DATABASE {}", db_name);
            need_create = true
        }

        drop(conn);
        pool.disconnect().await?;

        // re connected to this database
        let pool = mysql_async::Pool::new(target_opt.as_ref());

        let db_conn = MysqlDBConn { db_name, pool };

        if need_create {
            db_conn.create_table().await?;
        }
        Ok(db_conn)
    }

    pub async fn close(self) -> Result<()> {
        Ok(self.pool.disconnect().await?)
    }

    async fn create_table(&self) -> Result<()> {
        println!("create table for {}", self.db_name);
        let mut conn = self.pool.get_conn().await?;

        // MetricsCounter
        let _ = CounterUnit::new_sql_table_opt().run(&mut conn).await?;
        // MetricsTimer
        let _ = TimerUnit::new_sql_table_opt().run(&mut conn).await?;
        // MetricsFlow
        let _ = FlowUnit::new_sql_table_opt().run(&mut conn).await?;

        drop(conn);
        Ok(())
    }

    pub(crate) async fn insert<UnitType>(&self, insert_data: Vec<UnitType>) -> Result<()>
    where
        UnitType: SqlTable,
    {
        let mut conn = self.pool.get_conn().await?;

        UnitType::insert_table_opt()
            .with(
                insert_data
                    .iter()
                    .map(|_d| _d.to_params())
                    .collect::<Vec<Params>>(),
            )
            .batch(&mut conn)
            .await?;

        drop(conn);

        Ok(())
    }
}

#[cfg(test)]
mod test {

    use super::*;

    async fn test_create_conn() -> Result<MysqlDBConn> {
        let test_url_opt = String::from(crate::config::CONSUMER_MYSQL_URL);
        let conn = MysqlDBConn::new(test_url_opt, String::from("test_db")).await?;
        Ok(conn)
    }

    async fn test_create_insert() -> Result<()> {
        let conn = test_create_conn().await?;
        let d = CounterUnit::rand();
        let mut vd = Vec::new();
        vd.push(d);

        conn.insert(vd).await?;
        Ok(())
    }

    #[test]
    fn test_conn() {
        tokio_test::block_on(test_create_conn()).unwrap();
    }

    #[test]
    fn test_insert() {
        tokio_test::block_on(test_create_insert()).unwrap();
    }
}
