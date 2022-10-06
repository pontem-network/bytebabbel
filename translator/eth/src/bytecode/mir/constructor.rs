use std::collections::HashMap;

use primitive_types::U256;

use crate::bytecode::mir::ir::expression::Expression;
use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::{SType, Value};
use crate::bytecode::mir::translation::variables::Variables;
use crate::Mir;

pub fn make_constructor(store: HashMap<U256, U256>) -> Mir {
    let mut mir = Mir::default();
    let mut variables = Variables::new(vec![SType::Signer]);
    mir.push(Statement::InitStorage(variables.borrow_param(0)));
    let store_var = variables.borrow(SType::Storage);
    mir.push(Statement::Assign(
        store_var,
        Expression::GetStore.ty(SType::Storage),
    ));

    let key_var = variables.borrow(SType::Num);
    let value_var = variables.borrow(SType::Num);
    for (key, value) in store {
        mir.push(Statement::Assign(
            key_var,
            Expression::Const(Value::from(key)).ty(SType::Num),
        ));
        mir.push(Statement::Assign(
            value_var,
            Expression::Const(Value::from(value)).ty(SType::Num),
        ));
        mir.push(Statement::SStore {
            storage: store_var,
            key: key_var,
            val: value_var,
        });
    }
    mir.push(Statement::Result(vec![]));
    mir.set_locals(variables.locals());
    mir
}
