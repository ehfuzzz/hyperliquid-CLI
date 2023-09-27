use hyperliquid::types::info::response::{AssetContext, Ctx};

pub fn asset_ctx<'a>(
    asset_ctxs: &'a Vec<AssetContext>,
    asset: &str,
) -> Result<Option<&'a Ctx>, anyhow::Error> {
    let universe = match asset_ctxs.get(0) {
        Some(AssetContext::Meta(universe)) => universe,
        _ => return Ok(None),
    };

    let position = universe
        .universe
        .iter()
        .position(|a| a.name.to_uppercase() == asset.to_uppercase())
        .expect("Asset not found");

    let ctxs = match asset_ctxs.get(1) {
        Some(AssetContext::Ctx(ctxs)) => ctxs,
        _ => return Ok(None),
    };

    let ctx = ctxs.get(position);

    Ok(ctx)
}
