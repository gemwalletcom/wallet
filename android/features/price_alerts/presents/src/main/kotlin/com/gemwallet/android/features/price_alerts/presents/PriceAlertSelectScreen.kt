package com.gemwallet.android.features.price_alerts.presents

import androidx.compose.runtime.Composable
import com.gemwallet.android.features.assets.presents.select.AssetSelectScreen
import com.gemwallet.android.features.assets.presents.select.assetSelectViewModel
import com.gemwallet.android.features.assets.viewmodels.select.AssetSelectViewModel
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
