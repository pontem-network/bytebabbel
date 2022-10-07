use std::collections::HashMap;

use crate::bytecode::loc::Loc;
use primitive_types::U256;

use crate::bytecode::mir::ir::expression::Expression;
use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::{SType, Value};
use crate::bytecode::mir::translation::variables::Variables;
use crate::Mir;

pub fn make_constructor(store: HashMap<U256, U256>) -> Mir {
    let mut mir = Mir::default();
    let mut variables = Variables::new(vec![SType::Signer]);
    let loc: Loc<()> = Loc::default();
    mir.push(loc.wrap(Statement::InitStorage(variables.borrow_param(0))));
    let store_var = variables.borrow(SType::Storage);
    mir.push(loc.wrap(Statement::Assign(
        store_var,
        loc.wrap(Expression::GetStore.ty(SType::Storage)),
    )));

    for (key, value) in store {
        mir.push(loc.wrap(Statement::SStore {
            storage: store_var,
            key: loc.wrap(Expression::Const(Value::from(key)).ty(SType::Num)),
            val: loc.wrap(Expression::Const(Value::from(value)).ty(SType::Num)),
        }));
    }
    mir.push(loc.wrap(Statement::Result(vec![])));
    mir.set_locals(variables.locals());
    mir
}
