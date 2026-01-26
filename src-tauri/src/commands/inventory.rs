use crate::models::inventory::{InventoryItem, InventoryResult};
use crate::services::inventory_scanner::InventoryScanner;

#[tauri::command]
pub async fn scan_inventory() -> Result<InventoryResult, String> {
    Ok(InventoryScanner::scan_all().await)
}

#[tauri::command]
pub async fn refresh_inventory_item(id: String) -> Result<InventoryItem, String> {
    InventoryScanner::refresh_item(&id)
        .await
        .ok_or_else(|| format!("Item not found: {}", id))
}
