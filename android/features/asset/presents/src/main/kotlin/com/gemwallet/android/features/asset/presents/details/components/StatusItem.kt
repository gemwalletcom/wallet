package com.gemwallet.android.features.asset.presents.details.components

import androidx.compose.foundation.Image
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ext.type
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoBottomSheet
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.compactIconSize
import com.gemwallet.android.ui.theme.smallIconSize
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetSubtype

internal fun LazyListScope.status(asset: Asset, rank: Int) {
    val verification = assetVerification(asset, rank) ?: return
    item {
        StatusItem(verification)
    }
}

@Composable
private fun StatusItem(verification: AssetVerification) {
    val infoSheetEntity = verification.infoSheetEntity()
    var showInfoSheet by remember { mutableStateOf(false) }

    PropertyItem(
        modifier = Modifier.clickable {
            showInfoSheet = true
        },
        title = {
            PropertyTitleText(
                text = R.string.transaction_status,
                info = infoSheetEntity
            )
        },
        data = {
            PropertyDataText(
                stringResource(verification.labelRes()),
                color = verification.color(),
                badge = {
                    DataBadgeChevron {
                        VerificationBadgeIcon(verification)
                    }
                }
            )
        },
        listPosition = ListPosition.Single,
    )

    if (showInfoSheet) {
        InfoBottomSheet(item = infoSheetEntity) {
            showInfoSheet = false
        }
    }
}

@Composable
private fun VerificationBadgeIcon(verification: AssetVerification) {
    Box(
        modifier = Modifier.size(smallIconSize),
        contentAlignment = Alignment.Center,
    ) {
        Image(
            painter = painterResource(verification.badgeIconRes()),
            contentDescription = null,
            modifier = Modifier.size(compactIconSize),
        )
    }
}

internal fun assetVerification(asset: Asset, rank: Int): AssetVerification? {
    if (asset.id.type() == AssetSubtype.NATIVE) {
        return null
    }

    return rank.getVerificationStatus()
}

internal enum class AssetVerification(val maxScore: Int) {
    Suspicious(5),
    Unverified(15),
}

private fun Int.getVerificationStatus(): AssetVerification? {
    return if (this <= AssetVerification.Suspicious.maxScore) {
        AssetVerification.Suspicious
    } else if (this <= AssetVerification.Unverified.maxScore) {
        AssetVerification.Unverified
    } else {
        null
    }
}
