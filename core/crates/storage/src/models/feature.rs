use diesel::prelude::*;
use primitives::FeaturePolicy;

use crate::sql_types::Feature;

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::features)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct FeatureRow {
    pub id: Feature,
    pub alpha2: String,
    pub is_enabled: bool,
}

impl FeatureRow {
    pub fn into_policy(self) -> FeaturePolicy {
        FeaturePolicy {
            feature: self.id.into(),
            country_code: self.alpha2,
            is_enabled: self.is_enabled,
        }
    }
}
