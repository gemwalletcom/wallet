use diesel::prelude::*;
use primitives::FeaturePolicy;

use crate::models::FeatureRow;
use crate::{DatabaseClient, DatabaseError};

pub trait FeaturesRepository {
    fn get_features(&mut self) -> Result<Vec<FeaturePolicy>, DatabaseError>;
}

impl FeaturesRepository for DatabaseClient {
    fn get_features(&mut self) -> Result<Vec<FeaturePolicy>, DatabaseError> {
        use crate::schema::features::dsl::*;
        let rows = features.select(FeatureRow::as_select()).load(&mut self.connection)?;
        Ok(rows.into_iter().map(FeatureRow::into_policy).collect())
    }
}
