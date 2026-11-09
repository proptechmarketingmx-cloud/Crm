use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActorType {
    User,
    DesktopApp,
    McpClient,
    Import,
    System,
}
