use crm_mcp::{
    default_runtime_status_json, handle_jsonrpc_once, help_message,
    load_runtime_config_from_optional_path, read_only_tool_catalog_json, run_stdio_loop,
    runtime_status_json, DEFAULT_STATUS_MESSAGE, HANDLE_JSONRPC_ONCE_FLAG, LIST_TOOLS_FLAG,
    PRINT_RUNTIME_STATUS_FLAG, PRINT_RUNTIME_STATUS_FROM_CONFIG_FLAG, PRINT_TOOL_CATALOG_FLAG,
    SERVE_STDIO_FROM_CONFIG_FLAG,
};

fn main() {
    let mut args = std::env::args();
    let program_name = args.next().unwrap_or_else(|| "crm-mcp".to_string());
    let args: Vec<String> = args.collect();

    match args.as_slice() {
        [] => println!("{DEFAULT_STATUS_MESSAGE}"),
        [flag] if flag == PRINT_TOOL_CATALOG_FLAG || flag == LIST_TOOLS_FLAG => {
            let catalog_json =
                read_only_tool_catalog_json().expect("offline MCP catalog should serialize");
            println!("{catalog_json}");
        }
        [flag] if flag == PRINT_RUNTIME_STATUS_FLAG => {
            let status_json =
                default_runtime_status_json().expect("offline MCP runtime status should serialize");
            println!("{status_json}");
        }
        [flag, path] if flag == PRINT_RUNTIME_STATUS_FROM_CONFIG_FLAG => {
            let config = match load_runtime_config_from_optional_path(path) {
                Ok(config) => config,
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(2);
                }
            };
            let status_json =
                runtime_status_json(&config).expect("offline MCP runtime status should serialize");
            println!("{status_json}");
        }
        [flag, raw_json] if flag == HANDLE_JSONRPC_ONCE_FLAG => {
            let response_json = handle_jsonrpc_once(raw_json)
                .expect("metadata-only JSON-RPC response should serialize");
            if let Some(response_json) = response_json {
                println!("{response_json}");
            }
        }
        [flag, path] if flag == SERVE_STDIO_FROM_CONFIG_FLAG => {
            let config = match load_runtime_config_from_optional_path(path) {
                Ok(config) => config,
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(2);
                }
            };
            let stdin = std::io::stdin();
            let stdout = std::io::stdout();
            if let Err(error) = run_stdio_loop(&config, stdin.lock(), stdout.lock()) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        [flag] if flag == "--help" || flag == "-h" => {
            println!("{}", help_message(&program_name));
        }
        _ => {
            eprintln!(
                "Unsupported crm-mcp arguments. Use {PRINT_TOOL_CATALOG_FLAG} or {LIST_TOOLS_FLAG} to print the offline SDK-backed read-only catalog, {PRINT_RUNTIME_STATUS_FLAG} to print the disabled runtime guard status, {PRINT_RUNTIME_STATUS_FROM_CONFIG_FLAG} <path> to load JSON config metadata and print non-serving runtime guard status, {HANDLE_JSONRPC_ONCE_FLAG} <json> to handle one metadata-only JSON-RPC request, or {SERVE_STDIO_FROM_CONFIG_FLAG} <path> to attempt the disabled-by-default local stdio metadata loop. MCP server/runtime startup is not implemented."
            );
            std::process::exit(2);
        }
    }
}
