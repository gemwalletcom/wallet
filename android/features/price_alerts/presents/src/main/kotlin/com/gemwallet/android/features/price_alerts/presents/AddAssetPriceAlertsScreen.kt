package com.gemwallet.android.features.price_alerts.presents

import androidx.compose.runtime.Composable
import com.gemwallet.android.features.assets.presents.select.SelectAssetScreen
import com.gemwallet.android.features.assets.presents.select.selectAssetViewModel
import com.gemwallet.android.features.assets.viewmodels.select.SelectAssetViewModel
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemSelectAssetType

@Composable
fun AddAssetPriceAlertsScreen(onCancel: () -> Unit, onSelect: ((AssetId) -> Unit)? = null, viewModel: SelectAssetViewModel = selectAssetViewModel(GemSelectAssetType.PriceAlert)) {
    SelectAssetScreen(
        onCancel = onCancel,
        onSelect = onSelect,
        viewModel = viewModel,
    )
}
