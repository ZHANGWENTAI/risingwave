use crate::array::{ArrayBuilder, ArrayImpl, ArrayRef, DataChunk, I32ArrayBuilder};
use crate::error::Result;
use crate::expr::{BoxedExpression, Expression};
use crate::types::{DataType, DataTypeRef, Int32Type};
use log::debug;
use std::sync::Arc;

/// `PG_SLEEP` sleeps on current session for given duration (double precision in seconds),
/// and returns `NULL` for all inputs.
///
/// Note that currently `PG_SLEEP` accepts decimals as arguments, which is not compatible
/// with Postgres. The reason for this is that Calcite always converts float/double to
/// decimal, but not vice versa.
#[derive(Debug)]
pub struct PgSleepExpression {
    child_expr: BoxedExpression,
    return_type: DataTypeRef,
}

impl PgSleepExpression {
    pub fn new(child_expr: BoxedExpression) -> Self {
        PgSleepExpression {
            child_expr,
            return_type: Int32Type::create(true),
        }
    }
}

impl Expression for PgSleepExpression {
    fn return_type(&self) -> &dyn DataType {
        &*self.return_type
    }

    fn return_type_ref(&self) -> DataTypeRef {
        self.return_type.clone()
    }

    fn eval(&mut self, input: &DataChunk) -> Result<ArrayRef> {
        use num_traits::ToPrimitive;
        use std::time::Duration;

        let child_result = self.child_expr.eval(input)?;
        let mut array_builder = I32ArrayBuilder::new(input.cardinality())?;
        for datum in child_result.iter() {
            if let Some(duration) = datum {
                // Postgres accepts double precisions, but Calcite likes decimals
                let duration_secs = duration.into_decimal().to_f64().unwrap();
                if duration_secs > 0.0 {
                    let duration_ms = (duration_secs * 1000.0) as u64;
                    debug!("pg_sleep() for {} ms", duration_ms);
                    std::thread::sleep(Duration::from_millis(duration_ms));
                }
            }
            array_builder.append_null()?;
        }
        let array = array_builder.finish()?;
        Ok(Arc::new(ArrayImpl::from(array)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::array::column::Column;
    use crate::array::DecimalArrayBuilder;
    use crate::expr::InputRefExpression;
    use crate::types::DecimalType;
    use rust_decimal::prelude::FromStr;
    use rust_decimal::Decimal;

    #[test]
    fn test_pg_sleep() -> Result<()> {
        let decimal_type = DecimalType::create(true, 10, 2)?;
        let mut expr =
            PgSleepExpression::new(Box::new(InputRefExpression::new(decimal_type.clone(), 0)));

        let input_array = {
            let mut builder = DecimalArrayBuilder::new(3)?;
            builder.append(Some(Decimal::from_str("0.1").unwrap()))?;
            builder.append(Some(Decimal::from_str("-0.1").unwrap()))?;
            builder.append(None)?;
            builder.finish()?
        };

        let input_chunk = DataChunk::new(
            vec![Column::new(
                Arc::new(ArrayImpl::Decimal(input_array)),
                decimal_type,
            )],
            None,
        );
        let result_array = expr.eval(&input_chunk).unwrap();
        assert_eq!(3, result_array.len());
        for i in 0..3 {
            assert!(result_array.value_at(i).is_none());
        }
        Ok(())
    }
}