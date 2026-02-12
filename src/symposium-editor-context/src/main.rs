use anyhow::Result;
use sacp::Component;
use symposium_editor_context::EditorContextComponent;

#[tokio::main]
async fn main() -> Result<()> {
    EditorContextComponent
        .serve(sacp_tokio::Stdio::new())
        .await?;
    Ok(())
}
