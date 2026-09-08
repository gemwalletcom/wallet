package com.gemwallet.android.features.activities.presents.details.components

import com.gemwallet.android.ui.components.image.iconModel
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.domains.transaction.values.TransactionDetailsValue
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator16
import com.gemwallet.android.ui.components.color
import com.gemwallet.android.ui.components.statusLabelRes
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.Spacer8
import com.wallet.core.primitives.Asset

@Composable
fun TransactionStatusProperty(asset: Asset, property: TransactionDetailsValue.Status, position: ListPosition) {
    val color = property.status.tone.color()
    val showsStatusProgress = property.status.showsProgress

    PropertyItem(
        title = {
            PropertyTitleText(R.string.transaction_status, info = InfoSheetEntity.TransactionInfo(icon = asset.iconModel(), state = property.data, tone = property.status.tone))
        },
        data = {
            PropertyDataText(
                text = stringResource(id = property.data.statusLabelRes()),
                color = color,
                badge = if (showsStatusProgress) {
                    {
                        Spacer8()
                        CircularProgressIndicator16(color = color)
                    }
                } else {
                    null
                },
            )
        },
        listPosition = position,
    )
}
