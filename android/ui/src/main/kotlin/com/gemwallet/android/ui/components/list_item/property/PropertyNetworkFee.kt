package com.gemwallet.android.ui.components.list_item.property

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.clickable
import com.gemwallet.android.ui.models.ListPosition

@Composable
fun PropertyNetworkFee(
    networkTitle: String,
    networkSymbol: String,
    feeCrypto: String,
    feeFiat: String,
    variantsAvailable: Boolean = false,
    showedCryptoAmount: Boolean = false,
    showFeeAssetSymbol: Boolean = false,
    onClick: (() -> Unit)? = null,
) {
    PropertyItem(
        modifier = if (variantsAvailable && onClick != null) {
            Modifier.clickable(onClick = onClick)
        } else {
            Modifier
        },
        title = {
            PropertyTitleText(R.string.transfer_network_fee, info = InfoSheetEntity.NetworkFeeInfo(networkTitle, networkSymbol))
        },
        data = {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Column(horizontalAlignment = Alignment.End) {
                    val hasFiatAmount = feeFiat.isNotEmpty()
                    val primary = if (showedCryptoAmount || !hasFiatAmount) feeCrypto else feeFiat
                    Row(horizontalArrangement = Arrangement.End) { PropertyDataText(primary) }
                    val secondary = when {
                        showedCryptoAmount -> feeFiat
                        showFeeAssetSymbol && hasFiatAmount -> networkSymbol
                        else -> ""
                    }
                    if (secondary.isNotEmpty()) {
                        Row(horizontalArrangement = Arrangement.End) { PropertyDataText(secondary) }
                    }
                }
                if (variantsAvailable) {
                    DataBadgeChevron()
                }
            }
        },
        listPosition = ListPosition.Single,
    )
}
