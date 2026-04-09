package com.gemwallet.android.features.settings.price_alerts.presents

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.assetPriceSupport
import com.gemwallet.android.features.asset_select.presents.views.AssetSelectScreen
import com.gemwallet.android.features.asset_select.presents.views.getAssetBadge
import com.gemwallet.android.features.settings.price_alerts.viewmodels.PriceAlertsSelectViewModel
import com.wallet.core.primitives.AssetId

@Composable
fun PriceAlertSelectScreen(
    onCancel: () -> Unit,
    onSelect: ((AssetId) -> Unit)? = null,
    viewModel: PriceAlertsSelectViewModel = hiltViewModel()
) {
    AssetSelectScreen(
        title = stringResource(id = R.string.assets_select_asset),
        titleBadge = ::getAssetBadge,
        itemSupport = { assetPriceSupport(it.price) },
        onCancel = onCancel,
        onSelect = onSelect,
        viewModel = viewModel,
    )
}
