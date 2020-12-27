use super::{Balance, Person, Squad, Transaction};
use crate::db::{
    models,
    schema::{balance, node, person, squad, txn},
    Pool,
};
use diesel::prelude::*;
use tokio_diesel::*;
use uuid::Uuid;

#[async_graphql::Interface(field(name = "id", type = "String"))]
pub enum Node {
    Person(Person),
    Squad(Squad),
    Balance(Balance),
    Transaction(Transaction),
}

impl Node {
    pub async fn by_uid(pool: &Pool, uid: Uuid) -> AsyncResult<Node> {
        pool.transaction(move |conn| {
            let node = node::table
                .filter(node::uid.eq(uid))
                .get_result::<models::Node>(conn)?;

            match node.node_type {
                models::NodeType::Person => {
                    let detail = person::table
                        .filter(person::node_id.eq(node.id))
                        .get_result::<models::PersonDetail>(conn)?;

                    Ok(Node::Person(Person {
                        model: models::Person { node, detail }.into(),
                    }))
                }
                models::NodeType::Squad => {
                    let detail = squad::table
                        .filter(squad::node_id.eq(node.id))
                        .get_result::<models::SquadDetail>(conn)?;

                    Ok(Node::Squad(Squad {
                        model: models::Squad { node, detail }.into(),
                    }))
                }
                models::NodeType::Balance => {
                    let detail = balance::table
                        .filter(balance::node_id.eq(node.id))
                        .get_result::<models::BalanceDetail>(conn)?;

                    Ok(Node::Balance(Balance {
                        model: models::Balance { node, detail }.into(),
                    }))
                }
                models::NodeType::Txn => {
                    let detail = txn::table
                        .filter(txn::node_id.eq(node.id))
                        .get_result::<models::TransactionDetail>(conn)?;

                    Ok(Node::Transaction(Transaction {
                        model: models::Transaction { node, detail }.into(),
                    }))
                }
            }
        })
        .await
    }
}
