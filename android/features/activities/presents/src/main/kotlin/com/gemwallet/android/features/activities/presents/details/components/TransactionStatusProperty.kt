package com.gemwallet.android.features.activities.presents.details.components

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.domains.asset.getIconUrl
import com.gemwallet.android.domains.transaction.values.TransactionDetailsValue
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator16
import com.gemwallet.android.ui.components.showsStatusProgress
import com.gemwallet.android.ui.components.statusColor
import com.gemwallet.android.ui.components.statusLabelRes
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.Spacer8
import com.wallet.core.primitives.Asset

@Composable
fun TransactionStatusProperty(asset: Asset, property: TransactionDetailsValue.Status, position: ListPosition) {
    val color = property.data.statusColor()

    PropertyItem(
        title = {
            PropertyTitleText(R.string.transaction_status, info = InfoSheetEntity.TransactionInfo(icon = asset.getIconUrl(), state = property.data))
        },
        data = {
            PropertyDataText(
                text = stringResource(id = property.data.statusLabelRes()),
                color = color,
                badge = {
                    Spacer8()
                    if (property.data.showsStatusProgress()) {
                        CircularProgressIndicator16(color = color)
                    }
                },
            )
        },
        listPosition = position,
    )
}
