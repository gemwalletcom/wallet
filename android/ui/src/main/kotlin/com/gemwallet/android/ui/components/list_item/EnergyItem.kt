package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.titleRes
import com.gemwallet.android.ui.models.ListPosition
import com.wallet.core.primitives.BalanceMetadata
import uniffi.gemstone.balanceResourceRows

fun LazyListScope.energyItem(balanceMetadata: BalanceMetadata?) {
    val rows = balanceResourceRows(balanceMetadata?.toGem())
    if (rows.isEmpty()) return
    item {
        SubheaderItem(R.string.asset_resources)
        rows.forEachIndexed { index, row ->
            ListItem(
                model = ListItemModel(title = stringResource(row.resource.titleRes()), subtitle = row.text),
                listPosition = if (index == 0) ListPosition.First else ListPosition.Last,
            )
        }
    }
}
