package com.gemwallet.android.features.assets.presents.select

import androidx.compose.runtime.Composable
import com.gemwallet.android.features.assets.viewmodels.select.SelectAssetViewModel
import com.gemwallet.android.ui.models.actions.CancelAction
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemSelectAssetType

@Composable
fun SelectBuyScreen(cancelAction: CancelAction, onSelect: ((AssetId) -> Unit)?, viewModel: SelectAssetViewModel = selectAssetViewModel(GemSelectAssetType.Buy)) {
    SelectAssetScreen(
        onSelect = onSelect,
        onSelectRecent = onSelect,
        onCancel = { cancelAction.invoke() },
        viewModel = viewModel,
    )
}
