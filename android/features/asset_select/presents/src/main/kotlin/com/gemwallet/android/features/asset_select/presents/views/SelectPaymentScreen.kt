package com.gemwallet.android.features.asset_select.presents.views

import androidx.compose.runtime.Composable
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import com.gemwallet.android.features.asset_select.viewmodels.PaymentSelectViewModel
import com.wallet.core.primitives.AssetId

@Composable
fun SelectPaymentScreen(onCancel: () -> Unit, onSelect: (AssetId) -> Unit, viewModel: PaymentSelectViewModel = hiltViewModel()) {
    AssetSelectScreen(
        onCancel = onCancel,
        onSelect = onSelect,
        onSelectRecent = onSelect,
        viewModel = viewModel,
    )
}
