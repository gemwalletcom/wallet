package com.gemwallet.android.features.asset.presents.details.components

import androidx.compose.foundation.lazy.LazyListScope
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.features.asset.viewmodels.details.models.AssetInfoUIModel

internal fun LazyListScope.balancesHeader(model: AssetInfoUIModel.AccountInfoUIModel) {
    if (model.balances.isEmpty()) {
        return
    }
    item {
        SubheaderItem(R.string.asset_balances)
    }
}