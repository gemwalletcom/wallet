package com.gemwallet.android.features.asset_select.presents.views

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import com.gemwallet.android.features.asset_select.viewmodels.SendSelectViewModel
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain

@Composable
fun SelectSendScreen(
    onCancel: () -> Unit,
    onSelect: ((AssetId) -> Unit)?,
    chains: List<Chain> = emptyList(),
    viewModel: SendSelectViewModel = hiltViewModel()
) {
    LaunchedEffect(chains) {
        viewModel.setChainFilter(chains)
    }

    AssetSelectScreen(
        onSelect = onSelect,
        onSelectRecent = onSelect,
        onCancel = onCancel,
        viewModel = viewModel,
    )
}
