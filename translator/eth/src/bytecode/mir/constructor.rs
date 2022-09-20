use crate::bytecode::mir::ir::expression::Expression;
use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::{SType, Value};
use crate::bytecode::mir::translation::variables::Variables;
use crate::Mir;
use primitive_types::U256;
use std::collections::HashMap;

pub fn make_constructor(store: HashMap<U256, U256>) -> Mir {
    let mut mir = Mir::default();
    let mut variables = Variables::new(vec![SType::Signer]);
    mir.add_statement(Statement::InitStorage(variables.borrow_param(0)));
    let store_var = variables.borrow_global(SType::Storage);
    mir.add_statement(Statement::Assign(
        store_var,
        Expression::GetStore.ty(SType::Storage),
    ));
    for (key, value) in store {
        mir.add_statement(Statement::SStore {
            storage: store_var,
            key: Expression::Const(Value::from(key)).ty(SType::Num),
            val: Expression::Const(Value::from(value)).ty(SType::Num),
        });
    }
    mir.add_statement(Statement::Result(vec![]));
    mir.set_locals(variables.locals());
    mir
}
