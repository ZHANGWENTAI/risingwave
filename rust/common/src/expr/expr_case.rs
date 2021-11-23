use crate::array::{ArrayRef, DataChunk};
use crate::error::Result;
use crate::expr::{BoxedExpression, DataType, Expression};
use crate::types::DataTypeRef;
use itertools::Itertools;

pub struct WhenClause {
    pub when: BoxedExpression,
    pub then: BoxedExpression,
}

impl WhenClause {
    pub fn new(when: BoxedExpression, then: BoxedExpression) -> Self {
        WhenClause { when, then }
    }
}

pub struct CaseExpression {
    return_type: DataTypeRef,
    when_clauses: Vec<WhenClause>,
    else_clause: Option<BoxedExpression>,
}

impl CaseExpression {
    pub fn new(
        return_type: DataTypeRef,
        when_clauses: Vec<WhenClause>,
        else_clause: Option<BoxedExpression>,
    ) -> Self {
        Self {
            return_type,
            when_clauses,
            else_clause,
        }
    }
}

impl Expression for CaseExpression {
    fn return_type(&self) -> &dyn DataType {
        &*self.return_type
    }
    fn return_type_ref(&self) -> DataTypeRef {
        self.return_type.clone()
    }
    fn eval(&mut self, input: &DataChunk) -> Result<ArrayRef> {
        let mut els = self
            .else_clause
            .as_deref_mut()
            .map(|else_clause| else_clause.eval(input).unwrap());
        let when_thens = self
            .when_clauses
            .iter_mut()
            .map(|when_clause| {
                (
                    when_clause.when.eval(input).unwrap(),
                    when_clause.then.eval(input).unwrap(),
                )
            })
            .collect_vec();
        let mut output_array = self
            .return_type_ref()
            .create_array_builder(input.capacity())?;
        for idx in 0..input.capacity() {
            let t = if let Some((_, t)) = when_thens
                .iter()
                .map(|(w, t)| (w.value_at(idx), t.value_at(idx)))
                .find(|(w, _)| *w.unwrap().into_scalar_impl().as_bool())
            {
                Some(t.unwrap().into_scalar_impl())
            } else {
                els.as_mut()
                    .map(|e| e.value_at(idx).unwrap().into_scalar_impl())
            };
            output_array.append_datum(&t)?;
        }
        Ok(output_array.finish()?.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::array::{column::Column, PrimitiveArray};
    use crate::expr::{
        expr_binary_nonnull::new_binary_expr, InputRefExpression, LiteralExpression,
    };
    use crate::types::{BoolType, Float32Type, Int32Type, Scalar};
    use risingwave_pb::expr::expr_node::Type as ProstExprType;
    use std::sync::Arc;

    fn create_column_i32(vec: &[Option<i32>]) -> Result<Column> {
        let array = PrimitiveArray::from_slice(vec).map(|x| Arc::new(x.into()))?;
        let data_type = Int32Type::create(false);
        Ok(Column::new(array, data_type))
    }

    #[test]
    fn test_searched_case() {
        let ret_type = Float32Type::create(false);
        // when x <= 2 then 3.1
        let when_clauses = vec![WhenClause::new(
            new_binary_expr(
                ProstExprType::LessThanOrEqual,
                BoolType::create(false),
                Box::new(InputRefExpression::new(Int32Type::create(false), 0)),
                Box::new(LiteralExpression::new(
                    Float32Type::create(false),
                    Some(2f32.to_scalar_value()),
                )),
            ),
            Box::new(LiteralExpression::new(
                Float32Type::create(false),
                Some(3.1f32.to_scalar_value()),
            )),
        )];
        // else 4.1
        let els = Box::new(LiteralExpression::new(
            Float32Type::create(false),
            Some(4.1f32.to_scalar_value()),
        ));
        let mut searched_case_expr = CaseExpression::new(ret_type, when_clauses, Some(els));
        let col = create_column_i32(&[Some(1), Some(2), Some(3), Some(4), Some(5)]).unwrap();
        let input = DataChunk::builder().columns([col].to_vec()).build();
        let output = searched_case_expr.eval(&input).unwrap();
        assert_eq!(output.datum_at(0), Some(3.1f32.to_scalar_value()));
        assert_eq!(output.datum_at(1), Some(3.1f32.to_scalar_value()));
        assert_eq!(output.datum_at(2), Some(4.1f32.to_scalar_value()));
        assert_eq!(output.datum_at(3), Some(4.1f32.to_scalar_value()));
        assert_eq!(output.datum_at(4), Some(4.1f32.to_scalar_value()));
    }
}