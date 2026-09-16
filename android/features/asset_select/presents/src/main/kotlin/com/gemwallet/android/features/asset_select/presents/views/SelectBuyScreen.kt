package com.gemwallet.android.features.asset_select.presents.views

import androidx.compose.runtime.Composable
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.features.asset_select.viewmodels.BuySelectViewModel
import com.wallet.core.primitives.AssetId

@Composable
fun SelectBuyScreen(
    cancelAction: CancelAction,
    onSelect: ((AssetId) -> Unit)?,
    viewModel: BuySelectViewModel = hiltViewModel()
) {
    AssetSelectScreen(
        onSelect = onSelect,
        onSelectRecent = onSelect,
        onCancel = { cancelAction.invoke() },
        viewModel = viewModel,
    )
}
