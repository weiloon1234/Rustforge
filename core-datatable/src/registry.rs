use crate::executor::execute_datatable;
use crate::traits::AutoDataTable;
use crate::types::{DataTableContext, DataTableDescribe, DataTableExecution, DataTableInput};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

#[async_trait]
pub trait DynDataTable: Send + Sync {
    fn model_key(&self) -> &'static str;

    fn describe(&self, ctx: &DataTableContext) -> DataTableDescribe;

    async fn execute(
        &self,
        input: &DataTableInput,
        ctx: &DataTableContext,
    ) -> Result<DataTableExecution>;
}

struct AutoDataTableEntry<T: AutoDataTable> {
    table: T,
}

#[async_trait]
impl<T> DynDataTable for AutoDataTableEntry<T>
where
    T: AutoDataTable,
{
    fn model_key(&self) -> &'static str {
        self.table.model_key()
    }

    fn describe(&self, ctx: &DataTableContext) -> DataTableDescribe {
        self.table.describe(ctx)
    }

    async fn execute(
        &self,
        input: &DataTableInput,
        ctx: &DataTableContext,
    ) -> Result<DataTableExecution> {
        execute_datatable(&self.table, input, ctx).await
    }
}

#[derive(Default)]
pub struct DataTableRegistry {
    tables: HashMap<String, Arc<dyn DynDataTable>>,
}

impl DataTableRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<T>(&mut self, table: T)
    where
        T: AutoDataTable,
    {
        let key = table.model_key().to_ascii_lowercase();
        self.tables
            .insert(key, Arc::new(AutoDataTableEntry { table }));
    }

    pub fn register_dyn(&mut self, table: Arc<dyn DynDataTable>) {
        let key = table.model_key().to_ascii_lowercase();
        self.tables.insert(key, table);
    }

    pub fn get(&self, model: &str) -> Option<Arc<dyn DynDataTable>> {
        self.tables.get(&model.to_ascii_lowercase()).cloned()
    }

    pub fn describe(&self, model: &str, ctx: &DataTableContext) -> Result<DataTableDescribe> {
        let table = self.get(model).ok_or_else(|| {
            anyhow::anyhow!("Unknown datatable model '{}': not registered", model)
        })?;
        Ok(table.describe(ctx))
    }

    pub async fn execute(
        &self,
        input: &DataTableInput,
        ctx: &DataTableContext,
    ) -> Result<DataTableExecution> {
        let model = input
            .model
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("Missing datatable model key"))?;
        let table = self.get(model).ok_or_else(|| {
            anyhow::anyhow!("Unknown datatable model '{}': not registered", model)
        })?;
        table.execute(input, ctx).await
    }
}
