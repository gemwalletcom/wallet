use chrono::NaiveDateTime;
use diesel::deserialize::{self, FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::pg::{Pg, PgValue};
use diesel::prelude::*;
use diesel::serialize::{self, Output, ToSql};
use primitives::WebhookKind;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::str::FromStr;
use strum::{AsRefStr, EnumString};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, AsExpression, FromSqlRow, AsRefStr, EnumString, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[diesel(sql_type = crate::schema::sql_types::ApiClientScope)]
pub enum ApiClientScope {
    DevicesRead,
    DevicesSubscriptionsRead,
    DevicesTransactionsRead,
    WebhooksTransactions,
    WebhooksSupport,
    WebhooksFiat,
}

impl ApiClientScope {
    pub fn webhook(kind: WebhookKind) -> Self {
        match kind {
            WebhookKind::Transactions => Self::WebhooksTransactions,
            WebhookKind::Support => Self::WebhooksSupport,
            WebhookKind::Fiat => Self::WebhooksFiat,
        }
    }
}

impl FromSql<crate::schema::sql_types::ApiClientScope, Pg> for ApiClientScope {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        let value = std::str::from_utf8(bytes.as_bytes())?;
        Self::from_str(value).map_err(|error| format!("Invalid api client scope: {error}").into())
    }
}

impl ToSql<crate::schema::sql_types::ApiClientScope, Pg> for ApiClientScope {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        out.write_all(self.as_ref().as_bytes())?;
        Ok(serialize::IsNull::No)
    }
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::api_clients)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ApiClientRow {
    pub id: i32,
    pub name: String,
    pub secret: String,
    pub enabled: bool,
    pub updated_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::api_clients)]
pub(crate) struct NewApiClientRow {
    pub(crate) name: String,
}

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::api_client_scopes)]
pub(crate) struct NewApiClientScopeRow {
    pub(crate) client_id: i32,
    pub(crate) scope: ApiClientScope,
    pub(crate) resource: String,
}

#[derive(Debug, Clone)]
pub enum ApiClientResource {
    Global,
    WebhookSender(String),
}

impl ApiClientResource {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Global => "",
            Self::WebhookSender(sender) => sender,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ApiClientGrant {
    pub client_name: String,
    pub scope: ApiClientScope,
    pub resource: ApiClientResource,
}
