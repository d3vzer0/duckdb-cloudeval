use duckdb_loadable_macros::duckdb_entrypoint_c_api_v2;
use duckdb_neo::{
    DuckDBType, Error, Result,
    bind_arguments::BindArgument,
    connection::Context,
    data_chunk::{DataChunkRef, VectorCollection},
    extension::Extension,
    scalar::{ScalarCallbacks, ScalarFunctionBuilder},
    signature::{FunctionParameter, SignatureBuilder},
    table_function::{
        BindFunctionHandle, ExecColumnInfo, InitColumnData, TableFunctionBuilder,
        TableFunctionCallbacks,
    },
    vector::{Unknown, Vector},
};
use std::sync::atomic::{AtomicBool, Ordering};

struct EchoScalar;

impl ScalarCallbacks for EchoScalar {
    type BindData = ();
    type InitData = ();

    fn exec(
        &self,
        _bind_data: Option<&Self::BindData>,
        _init_data: Option<&mut Self::InitData>,
        _context: &Context,
        input: &mut VectorCollection,
        output: Vector<'_, Unknown>,
    ) -> Result<()> {
        let input = input.get_vector_at::<String>(0)?.reader()?;
        let mut output = output.cast::<String>()?;
        output.set_size(input.len())?;
        let mut writer = output.writer()?;

        for (i, value) in input.iter().enumerate() {
            let value = value.map(|s| format!("🐤 {s} 🦀 {s}"));
            writer.set(i, value.as_deref())?;
        }
        Ok(())
    }
}

struct HelloVTab;

impl TableFunctionCallbacks for HelloVTab {
    type BindData = String;
    type GlobalState = AtomicBool;
    type LocalState = ();

    fn bind(
        &self,
        context: &Context,
        arguments: Vec<BindArgument>,
        bind: BindFunctionHandle<'_>,
    ) -> Result<Self::BindData> {
        bind.add_result_column("column0", &String::logical_type(context)?)?;
        let value = arguments[0]
            .value
            .as_ref()
            .ok_or_else(|| Error::invalid_input("rusty_quack requires a constant name"))?;
        Ok(value.get::<String>()?.unwrap_or_else(|| "NULL".to_owned()))
    }

    fn init_global_state(
        &self,
        _bind_data: &Self::BindData,
        _context: &Context,
        _columns: InitColumnData<'_>,
    ) -> Result<(Option<Self::GlobalState>, Option<usize>)> {
        Ok((Some(AtomicBool::new(false)), Some(1)))
    }

    fn exec(
        &self,
        name: &Self::BindData,
        global_state: Option<&Self::GlobalState>,
        _local_state: Option<&mut Self::LocalState>,
        _context: &Context,
        mut output: DataChunkRef<'_>,
        _columns: ExecColumnInfo<'_>,
    ) -> Result<()> {
        let done = global_state.ok_or_else(|| Error::internal("Missing rusty_quack scan state"))?;
        let mut vector = output.get_vector_at_mut::<String>(0)?;
        if done.swap(true, Ordering::Relaxed) {
            vector.set_size(0)?;
        } else {
            vector.set_size(1)?;
            vector
                .writer()?
                .set(0, Some(&format!("Rusty Quack {name} 🐥")))?;
        }
        Ok(())
    }
}

#[duckdb_entrypoint_c_api_v2(extension_name = "duckdb_cloudeval", api_version = "v2.0.0")]
pub fn extension_entrypoint(extension: &Extension) -> Result<()> {
    ScalarFunctionBuilder::new(
        "rusty_echo",
        SignatureBuilder::new()
            .param(FunctionParameter::new::<String>("name"))
            .return_type(String::logical_type(extension)?),
        EchoScalar,
    )
    .register(extension)?;
    TableFunctionBuilder::new(
        "rusty_quack",
        SignatureBuilder::new().param(FunctionParameter::new::<String>("name")),
        HelloVTab,
    )
    .register(extension)?;
    Ok(())
}
