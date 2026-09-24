package com.gemwallet.android.features.settings.price_alerts.presents

import androidx.compose.runtime.Composable
import com.gemwallet.android.features.asset_select.presents.views.AssetSelectScreen
import com.gemwallet.android.features.asset_select.presents.views.assetSelectViewModel
import com.gemwallet.android.features.asset_select.viewmodels.AssetSelectViewModel
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemSelectAssetType

@Composable
fun PriceAlertSelectScreen(onCancel: () -> Unit, onSelect: ((AssetId) -> Unit)? = null, viewModel: AssetSelectViewModel = assetSelectViewModel(GemSelectAssetType.PriceAlert)) {
    AssetSelectScreen(
        onCancel = onCancel,
        onSelect = onSelect,
        viewModel = viewModel,
    )
}
