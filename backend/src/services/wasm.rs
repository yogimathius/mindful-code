use crate::error::{AppError, Result};
use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, RwLock},
};
use tracing::{debug, error, info};
use wasmtime::{
    AsContextMut, Caller, Config, Engine, Instance, Linker, Module, Store, TypedFunc, WasmParams,
    WasmResults,
};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};

pub struct WasmPluginManager {
    engine: Engine,
    plugins: Arc<RwLock<HashMap<String, LoadedPlugin>>>,
}

struct LoadedPlugin {
    module: Module,
    plugin_info: PluginInfo,
    instance_count: u32,
}

#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub capabilities: Vec<String>,
}

pub struct WasmRuntime {
    store: Store<WasiCtx>,
    instance: Instance,
    plugin_name: String,
}

#[derive(Debug)]
pub struct PluginExecutionContext {
    pub user_id: uuid::Uuid,
    pub session_id: uuid::Uuid,
    pub data: serde_json::Value,
    pub permissions: Vec<String>,
}

impl WasmPluginManager {
    pub fn new() -> Result<Self> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.async_support(true);
        config.consume_fuel(true); // Enable fuel for execution limits
        
        let engine = Engine::new(&config)
            .map_err(|e| AppError::Wasm(format!("Failed to create WASM engine: {}", e)))?;

        info!("✅ WASM plugin manager initialized");

        Ok(Self {
            engine,
            plugins: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn load_plugin<P: AsRef<Path>>(&self, plugin_path: P, plugin_name: String) -> Result<()> {
        let plugin_bytes = tokio::fs::read(&plugin_path).await
            .map_err(|e| AppError::Wasm(format!("Failed to read plugin file: {}", e)))?;

        let module = Module::from_binary(&self.engine, &plugin_bytes)
            .map_err(|e| AppError::Wasm(format!("Failed to compile WASM module: {}", e)))?;

        // Extract plugin metadata
        let plugin_info = self.extract_plugin_info(&module, &plugin_name)?;

        info!("Loading plugin: {} v{}", plugin_info.name, plugin_info.version);
        debug!("Plugin capabilities: {:?}", plugin_info.capabilities);

        let loaded_plugin = LoadedPlugin {
            module,
            plugin_info,
            instance_count: 0,
        };

        let mut plugins = self.plugins.write().unwrap();
        plugins.insert(plugin_name.clone(), loaded_plugin);

        info!("✅ Plugin '{}' loaded successfully", plugin_name);
        Ok(())
    }

    pub async fn create_runtime(&self, plugin_name: &str) -> Result<WasmRuntime> {
        let plugins = self.plugins.read().unwrap();
        let plugin = plugins
            .get(plugin_name)
            .ok_or_else(|| AppError::Wasm(format!("Plugin '{}' not found", plugin_name)))?;

        // Create WASI context with restricted capabilities
        let wasi_ctx = WasiCtxBuilder::new()
            .inherit_stdio()
            .build();

        let mut store = Store::new(&self.engine, wasi_ctx);
        
        // Set execution limits
        store.set_fuel(1_000_000)?; // Limit execution fuel
        store.set_epoch_deadline(1); // Set timeout

        // Create linker with host functions
        let mut linker = Linker::new(&self.engine);
        wasmtime_wasi::add_to_linker(&mut linker, |s| s)
            .map_err(|e| AppError::Wasm(format!("Failed to add WASI to linker: {}", e)))?;

        // Add custom host functions
        self.add_host_functions(&mut linker)?;

        // Instantiate the module
        let instance = linker
            .instantiate(&mut store, &plugin.module)
            .map_err(|e| AppError::Wasm(format!("Failed to instantiate plugin: {}", e)))?;

        Ok(WasmRuntime {
            store,
            instance,
            plugin_name: plugin_name.to_string(),
        })
    }

    fn add_host_functions(&self, linker: &mut Linker<WasiCtx>) -> Result<()> {
        // Add logging function
        linker
            .func_wrap("mindful_code", "log", |caller: Caller<'_, WasiCtx>, level: i32, ptr: i32, len: i32| {
                let memory = match caller.get_export("memory") {
                    Some(wasmtime::Extern::Memory(mem)) => mem,
                    _ => return,
                };

                let data = match memory.data(&caller).get(ptr as usize..(ptr + len) as usize) {
                    Some(data) => data,
                    None => return,
                };

                let message = String::from_utf8_lossy(data);
                match level {
                    0 => tracing::debug!("[WASM] {}", message),
                    1 => tracing::info!("[WASM] {}", message),
                    2 => tracing::warn!("[WASM] {}", message),
                    3 => tracing::error!("[WASM] {}", message),
                    _ => tracing::info!("[WASM] {}", message),
                }
            })
            .map_err(|e| AppError::Wasm(format!("Failed to add log function: {}", e)))?;

        // Add time function
        linker
            .func_wrap("mindful_code", "get_timestamp", || {
                chrono::Utc::now().timestamp_millis()
            })
            .map_err(|e| AppError::Wasm(format!("Failed to add timestamp function: {}", e)))?;

        // Add data access function (with permission checks)
        linker
            .func_wrap(
                "mindful_code",
                "get_session_data",
                |_caller: Caller<'_, WasiCtx>, _session_id_ptr: i32| -> i32 {
                    // In a real implementation, this would:
                    // 1. Extract session_id from memory
                    // 2. Check permissions
                    // 3. Fetch data from database
                    // 4. Write result back to WASM memory
                    // For now, return success
                    1
                },
            )
            .map_err(|e| AppError::Wasm(format!("Failed to add data access function: {}", e)))?;

        info!("✅ Host functions added to WASM linker");
        Ok(())
    }

    fn extract_plugin_info(&self, module: &Module, fallback_name: &str) -> Result<PluginInfo> {
        // In a real implementation, this would parse plugin metadata
        // from custom sections or exported functions
        Ok(PluginInfo {
            name: fallback_name.to_string(),
            version: "1.0.0".to_string(),
            description: "WASM Plugin".to_string(),
            author: "Unknown".to_string(),
            capabilities: vec![
                "flow_analysis".to_string(),
                "data_processing".to_string(),
            ],
        })
    }

    pub fn get_loaded_plugins(&self) -> Vec<PluginInfo> {
        let plugins = self.plugins.read().unwrap();
        plugins
            .values()
            .map(|p| p.plugin_info.clone())
            .collect()
    }

    pub async fn unload_plugin(&self, plugin_name: &str) -> Result<()> {
        let mut plugins = self.plugins.write().unwrap();
        if plugins.remove(plugin_name).is_some() {
            info!("✅ Plugin '{}' unloaded", plugin_name);
            Ok(())
        } else {
            Err(AppError::Wasm(format!("Plugin '{}' not found", plugin_name)))
        }
    }
}

impl WasmRuntime {
    pub async fn execute_function<Params, Results>(
        &mut self,
        function_name: &str,
        params: Params,
    ) -> Result<Results>
    where
        Params: WasmParams,
        Results: WasmResults,
    {
        let func = self
            .instance
            .get_typed_func::<Params, Results>(&mut self.store, function_name)
            .map_err(|e| {
                AppError::Wasm(format!("Function '{}' not found: {}", function_name, e))
            })?;

        // Set epoch deadline for timeout
        self.store.set_epoch_deadline(1);

        let result = func
            .call(&mut self.store, params)
            .map_err(|e| AppError::Wasm(format!("Function execution failed: {}", e)))?;

        debug!(
            "Executed function '{}' in plugin '{}'",
            function_name, self.plugin_name
        );

        Ok(result)
    }

    pub async fn process_flow_data(
        &mut self,
        context: PluginExecutionContext,
    ) -> Result<serde_json::Value> {
        // Check if plugin has flow_analysis capability
        let serialized_context = serde_json::to_string(&context)
            .map_err(|e| AppError::Wasm(format!("Failed to serialize context: {}", e)))?;

        // In a real implementation, this would:
        // 1. Write context to WASM memory
        // 2. Call the plugin's process_flow_data function
        // 3. Read result from WASM memory
        // 4. Deserialize and return

        // For now, return a mock result
        Ok(serde_json::json!({
            "processed": true,
            "plugin": self.plugin_name,
            "timestamp": chrono::Utc::now().timestamp_millis(),
            "result": "Flow data processed successfully"
        }))
    }

    pub fn get_fuel_consumed(&self) -> Result<u64> {
        self.store
            .fuel_consumed()
            .ok_or_else(|| AppError::Wasm("Fuel tracking not enabled".to_string()))
    }

    pub fn add_fuel(&mut self, fuel: u64) -> Result<()> {
        self.store
            .add_fuel(fuel)
            .map_err(|e| AppError::Wasm(format!("Failed to add fuel: {}", e)))
    }
}

// Plugin development utilities
pub struct PluginBuilder {
    name: String,
    capabilities: Vec<String>,
    functions: Vec<String>,
}

impl PluginBuilder {
    pub fn new(name: String) -> Self {
        Self {
            name,
            capabilities: Vec::new(),
            functions: Vec::new(),
        }
    }

    pub fn add_capability(mut self, capability: String) -> Self {
        self.capabilities.push(capability);
        self
    }

    pub fn add_function(mut self, function: String) -> Self {
        self.functions.push(function);
        self
    }

    pub async fn build_template(&self, output_path: &Path) -> Result<()> {
        let template = format!(
            r#"
// Generated WASM plugin template for {}
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {{
    #[wasm_bindgen(js_namespace = mindful_code)]
    fn log(level: i32, message: &str);
    
    #[wasm_bindgen(js_namespace = mindful_code)]
    fn get_timestamp() -> i64;
}}

#[wasm_bindgen]
pub fn init() {{
    log(1, "Plugin {} initialized");
}}

// Capabilities: {:?}
{}

#[wasm_bindgen]
pub fn get_info() -> String {{
    serde_json::json!({{
        "name": "{}",
        "capabilities": {:?},
        "functions": {:?}
    }}).to_string()
}}
"#,
            self.name,
            self.name,
            self.capabilities,
            self.functions
                .iter()
                .map(|f| format!(
                    r#"
#[wasm_bindgen]
pub fn {}(data: &str) -> String {{
    log(1, &format!("Executing {}", data));
    // Your implementation here
    "{{\"success\": true}}".to_string()
}}
"#,
                    f, f
                ))
                .collect::<Vec<_>>()
                .join("\n"),
            self.name,
            self.capabilities,
            self.functions
        );

        tokio::fs::write(output_path, template)
            .await
            .map_err(|e| AppError::Wasm(format!("Failed to write template: {}", e)))?;

        info!("✅ Plugin template generated at {:?}", output_path);
        Ok(())
    }
}

impl Default for WasmPluginManager {
    fn default() -> Self {
        Self::new().expect("Failed to initialize WASM plugin manager")
    }
}