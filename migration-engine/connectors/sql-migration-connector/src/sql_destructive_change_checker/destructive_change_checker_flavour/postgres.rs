use super::DestructiveChangeCheckerFlavour;
use crate::{
    flavour::{PostgresFlavour, SqlFlavour},
    pair::Pair,
    sql_destructive_change_checker::{
        destructive_check_plan::DestructiveCheckPlan, unexecutable_step_check::UnexecutableStepCheck,
        warning_check::SqlMigrationWarningCheck,
    },
    sql_migration::{AlterColumn, ColumnTypeChange},
    sql_schema_differ::ColumnChanges,
};
use migration_connector::{BoxFuture, ConnectorResult};
use sql_schema_describer::walkers::ColumnWalker;

impl DestructiveChangeCheckerFlavour for PostgresFlavour {
    fn check_alter_column(
        &self,
        alter_column: &AlterColumn,
        columns: &Pair<ColumnWalker<'_>>,
        plan: &mut DestructiveCheckPlan,
        step_index: usize,
    ) {
        let AlterColumn {
            column_id: _,
            changes,
            type_change,
        } = alter_column;

        if changes.arity_changed() && columns.previous().arity().is_nullable() && columns.next().arity().is_required() {
            plan.push_unexecutable(
                UnexecutableStepCheck::MadeOptionalFieldRequired {
                    column: columns.previous().name().to_owned(),
                    table: columns.previous().table().name().to_owned(),
                },
                step_index,
            )
        }

        if changes.arity_changed() && !columns.previous().arity().is_list() && columns.next().arity().is_list() {
            plan.push_unexecutable(
                UnexecutableStepCheck::MadeScalarFieldIntoArrayField {
                    table: columns.previous().table().name().to_owned(),
                    column: columns.previous().name().to_owned(),
                },
                step_index,
            )
        }

        let previous_type = super::display_column_type(columns.previous, self.datamodel_connector());
        let next_type = super::display_column_type(columns.next, self.datamodel_connector());

        match type_change {
            None | Some(ColumnTypeChange::SafeCast) => (),
            Some(ColumnTypeChange::RiskyCast) => {
                plan.push_warning(
                    SqlMigrationWarningCheck::RiskyCast {
                        table: columns.previous().table().name().to_owned(),
                        column: columns.previous().name().to_owned(),
                        previous_type,
                        next_type,
                    },
                    step_index,
                );
            }
            Some(ColumnTypeChange::NotCastable) => {
                plan.push_warning(
                    SqlMigrationWarningCheck::NotCastable {
                        table: columns.previous().table().name().to_owned(),
                        column: columns.previous().name().to_owned(),
                        previous_type,
                        next_type,
                    },
                    step_index,
                );
            }
        };
    }

    fn check_drop_and_recreate_column(
        &self,
        columns: &Pair<ColumnWalker<'_>>,
        changes: &ColumnChanges,
        plan: &mut DestructiveCheckPlan,
        step_index: usize,
    ) {
        // Unexecutable drop and recreate.
        if changes.arity_changed()
            && columns.previous().arity().is_nullable()
            && columns.next().arity().is_required()
            && columns.next().default().is_none()
        {
            plan.push_unexecutable(
                UnexecutableStepCheck::AddedRequiredFieldToTable {
                    column: columns.previous().name().to_owned(),
                    table: columns.previous().table().name().to_owned(),
                },
                step_index,
            )
        } else if columns.next().arity().is_required() && columns.next().default().is_none() {
            plan.push_unexecutable(
                UnexecutableStepCheck::DropAndRecreateRequiredColumn {
                    column: columns.previous().name().to_owned(),
                    table: columns.previous().table().name().to_owned(),
                },
                step_index,
            )
        } else {
            // todo this is probably due to a not castable type change. we should give that info in the warning
            plan.push_warning(
                SqlMigrationWarningCheck::DropAndRecreateColumn {
                    column: columns.previous().name().to_owned(),
                    table: columns.previous().table().name().to_owned(),
                },
                step_index,
            )
        }
    }

    fn count_rows_in_table<'a>(&'a mut self, table_name: &'a str) -> BoxFuture<'a, ConnectorResult<i64>> {
        Box::pin(async move {
            let query = format!("SELECT COUNT(*) FROM \"{}\"", table_name);
            let result_set = self.query_raw(&query, &[]).await?;
            super::extract_table_rows_count(table_name, result_set)
        })
    }

    fn count_values_in_column<'a>(
        &'a mut self,
        (table, column): (&'a str, &'a str),
    ) -> BoxFuture<'a, ConnectorResult<i64>> {
        Box::pin(async move {
            let query = format!("SELECT COUNT(*) FROM \"{}\" WHERE \"{}\" IS NOT NULL", table, column);
            let result_set = self.query_raw(&query, &[]).await?;
            super::extract_column_values_count(result_set)
        })
    }
}
