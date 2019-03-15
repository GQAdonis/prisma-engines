use crate::{node_selector::NodeSelector, query_arguments::QueryArguments};
use prisma_common::PrismaResult;
use prisma_models::prelude::*;
use prisma_models::ScalarFieldRef;

pub trait DataResolver {
    fn get_node_by_where(
        &self,
        node_selector: &NodeSelector,
        selected_fields: &SelectedFields,
    ) -> PrismaResult<Option<SingleNode>>;

    fn get_nodes(
        &self,
        model: ModelRef,
        query_arguments: QueryArguments,
        selected_fields: SelectedFields,
    ) -> PrismaResult<ManyNodes>;

    fn get_related_nodes(
        &self,
        from_field: RelationFieldRef,
        from_node_ids: Vec<GraphqlId>,
        query_arguments: QueryArguments,
        selected_fields: SelectedFields,
    ) -> PrismaResult<ManyNodes>;

    fn get_scalar_list_values_by_node_ids(
        &self,
        list_field: ScalarFieldRef,
        node_ids: Vec<GraphqlId>,
    ) -> PrismaResult<Vec<ScalarListValues>>;

    fn count_by_model(&self, model: ModelRef, query_arguments: QueryArguments) -> PrismaResult<usize>;
    fn count_by_table(&self, database: &str, table: &str) -> PrismaResult<usize>;
}

pub struct ScalarListValues {
    pub node_id: GraphqlId,
    pub values: Vec<PrismaValue>,
}
