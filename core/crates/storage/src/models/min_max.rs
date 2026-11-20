use primitives::DateTime;

use crate::sql_types::DateTimeRow;

#[derive(Debug, Clone, Copy)]
pub(crate) struct DataPoint<T> {
    pub value: T,
    pub date: DateTime,
}

impl<T> From<(T, DateTimeRow)> for DataPoint<T> {
    fn from((value, date): (T, DateTimeRow)) -> Self {
        Self { value, date: date.into() }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct MinMax<T> {
    pub max: Option<DataPoint<T>>,
    pub min: Option<DataPoint<T>>,
}
