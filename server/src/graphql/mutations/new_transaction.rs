use super::super::nodes::{Squad, Transaction};
use crate::db::{
    models,
    schema::{balance, node, squad, txn, txn_part},
    Pool,
};
use async_graphql::{
    validators::{InputValueValidator, IntNonZero, ListMinLength},
    Error, Result, Value, ID,
};
use diesel::prelude::*;
use std::collections::HashMap;
use std::convert::TryFrom;
use tokio_diesel::*;
use uuid::Uuid;

struct ChangesSumToZero {}

impl InputValueValidator for ChangesSumToZero {
    fn is_valid(&self, value: &Value) -> Result<(), String> {
        if let Value::List(v) = value {
            let mut sum = 0;
            for detail in v {
                if let Value::Object(o) = detail {
                    if let Some(Value::Number(n)) = o.get("changeCents") {
                        if let Some(i) = n.as_i64().and_then(|x| i32::try_from(x).ok()) {
                            sum += i
                        }
                    }
                }
            }

            if sum != 0 {
                return Err(String::from("Changes must add up to zero"));
            }
        }

        Ok(())
    }
}

#[derive(async_graphql::InputObject)]
pub struct BalanceChangeDetail {
    pub balance_id: ID,
    #[graphql(validator(IntNonZero))]
    pub change_cents: i32,
}

pub struct ParsedBalanceChangeDetail {
    pub balance_uid: Uuid,
    pub change_cents: i32,
}

impl TryFrom<BalanceChangeDetail> for ParsedBalanceChangeDetail {
    type Error = Error;

    fn try_from(value: BalanceChangeDetail) -> Result<ParsedBalanceChangeDetail> {
        let balance_uid = Uuid::try_from(value.balance_id)?;

        Ok(ParsedBalanceChangeDetail {
            balance_uid,
            change_cents: value.change_cents,
        })
    }
}

#[derive(async_graphql::InputObject)]
pub struct NewTransactionInput {
    pub squad_id: ID,
    #[graphql(validator(and(ListMinLength(length = "1"), ChangesSumToZero)))]
    pub balance_changes_detail: Vec<BalanceChangeDetail>,
}

pub struct ParsedNewTransactionInput {
    pub squad_uid: Uuid,
    pub balance_changes_detail: HashMap<Uuid, i32>,
}

impl TryFrom<NewTransactionInput> for ParsedNewTransactionInput {
    type Error = Error;

    fn try_from(value: NewTransactionInput) -> Result<ParsedNewTransactionInput> {
        let squad_uid = Uuid::try_from(value.squad_id)?;

        Ok(ParsedNewTransactionInput {
            squad_uid,
            balance_changes_detail: value
                .balance_changes_detail
                .into_iter()
                .map(|b| {
                    let parsed = ParsedBalanceChangeDetail::try_from(b)?;

                    Ok((parsed.balance_uid, parsed.change_cents))
                })
                .collect::<Result<HashMap<Uuid, i32>>>()?,
        })
    }
}

#[derive(async_graphql::SimpleObject)]
pub struct NewTransactionPayload {
    pub squad: Squad,
    pub transaction: Transaction,
}

pub async fn new_transaction(
    pool: &Pool,
    input: ParsedNewTransactionInput,
) -> Result<NewTransactionPayload> {
    Ok(pool
        .transaction(move |conn| {
            let squad = node::table
                .inner_join(squad::table)
                .filter(node::uid.eq(input.squad_uid))
                .get_result::<models::Squad>(conn)?;

            let new_node = models::NewNode {
                uid: Uuid::new_v4(),
                node_type: models::NodeType::Txn,
            };

            let node = diesel::insert_into(node::table)
                .values(new_node)
                .get_result::<models::Node>(conn)?;

            let new_transaction = models::NewTransaction {
                node_id: node.id,
                squad_id: squad.detail.id,
            };

            let transaction = diesel::insert_into(txn::table)
                .values(&new_transaction)
                .get_result::<models::TransactionDetail>(conn)
                .map(|detail| models::Transaction { node, detail })?;

            let balance_uids = input.balance_changes_detail.keys().collect::<Vec<_>>();

            let balances = node::table
                .inner_join(balance::table)
                .filter(node::uid.eq_any(balance_uids))
                .get_results::<models::Balance>(conn)?;

            let new_parts = balances
                .iter()
                .map(|balance| {
                    Ok(models::NewTransactionPart {
                        txn_id: transaction.detail.id,
                        balance_id: balance.detail.id,
                        balance_change_cents: *input
                            .balance_changes_detail
                            .get(&balance.node.uid)
                            .expect("Internal error"),
                    })
                })
                .collect::<QueryResult<Vec<models::NewTransactionPart>>>()?;

            diesel::insert_into(txn_part::table)
                .values(new_parts)
                .execute(conn)?;

            Ok(NewTransactionPayload {
                squad: squad.into(),
                transaction: transaction.into(),
            })
        })
        .await?)
}
