use iced::Task;
use mysql::{Conn, OptsBuilder};

use super::{ConnectionParams, DriverError};

#[derive(Clone, Copy, Debug)]
pub struct MysqlDriver;

impl MysqlDriver {
    pub fn new() -> Self {
        Self
    }

    pub fn run_test_connection(
        &self,
        params: ConnectionParams,
    ) -> Task<Result<(), DriverError>> {
        Task::future(async move {
            let host = params.require_host()?;
            let port = params.require_port()?;
            let user = params.require_username()?;
            let password = params.password().unwrap_or_default();
            let database = params.database().unwrap_or_default();

            let mut builder = OptsBuilder::new()
                .ip_or_hostname(Some(host.to_string()))
                .tcp_port(port)
                .user(Some(user.to_string()))
                .pass(Some(password.to_string()));

            if !database.is_empty() {
                builder = builder.db_name(Some(database.to_string()));
            }

            let opts = mysql::Opts::from(builder);
            match Conn::new(opts) {
                Ok(mut conn) => {
                    // simple ping to ensure connection is valid
                    conn.ping().map_err(|err| DriverError::Connection(err.to_string()))?;
                    Ok(())
                }
                Err(err) => Err(DriverError::Connection(err.to_string())),
            }
        })
    }
}
