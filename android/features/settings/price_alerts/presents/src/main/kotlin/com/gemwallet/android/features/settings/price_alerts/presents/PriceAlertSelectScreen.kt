package com.gemwallet.android.features.settings.price_alerts.presents

import androidx.compose.runtime.Composable
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import com.gemwallet.android.features.asset_select.presents.views.AssetSelectScreen
import com.gemwallet.android.features.settings.price_alerts.viewmodels.PriceAlertsSelectViewModel
import com.wallet.core.primitives.AssetId

@Composable
fun PriceAlertSelectScreen(
    onCancel: () -> Unit,
    onSelect: ((AssetId) -> Unit)? = null,
    viewModel: PriceAlertsSelectViewModel = hiltViewModel()
) {
    AssetSelectScreen(
        onCancel = onCancel,
        onSelect = onSelect,
        viewModel = viewModel,
    )
}
