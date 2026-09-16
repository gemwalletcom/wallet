package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.lazy.LazyListScope
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.BalanceMetadata
import uniffi.gemstone.GemBalanceResource
import uniffi.gemstone.balanceResourceRows

fun LazyListScope.energyItem(balanceMetadata: BalanceMetadata?) {
    val rows = balanceResourceRows(balanceMetadata?.toGem())
    if (rows.isEmpty()) return
    item {
        SubheaderItem(R.string.asset_resources)
        rows.forEachIndexed { index, row ->
            PropertyItem(
                title = when (row.resource) {
                    GemBalanceResource.ENERGY -> R.string.stake_resource_energy
                    GemBalanceResource.BANDWIDTH -> R.string.stake_resource_bandwidth
                },
                data = row.text,
                listPosition = if (index == 0) ListPosition.First else ListPosition.Last,
            )
        }
    }
}