package com.gemwallet.android.features.assets.presents.select

import androidx.compose.runtime.Composable
import com.gemwallet.android.features.assets.viewmodels.select.SelectAssetViewModel
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemSelectAssetType

@Composable
fun SelectDepositScreen(onCancel: () -> Unit, onSelect: (AssetId) -> Unit, viewModel: SelectAssetViewModel = selectAssetViewModel(GemSelectAssetType.Deposit)) {
    SelectAssetScreen(
        onCancel = onCancel,
        onSelect = onSelect,
        onSelectRecent = onSelect,
        viewModel = viewModel,
    )
}
