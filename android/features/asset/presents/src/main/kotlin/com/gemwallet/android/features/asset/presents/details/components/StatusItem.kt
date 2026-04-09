package com.gemwallet.android.features.asset.presents.details.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ext.type
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.open
import com.gemwallet.android.ui.theme.Spacer16
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetSubtype
import com.gemwallet.android.AppUrl
import uniffi.gemstone.DocsUrl

internal fun LazyListScope.status(asset: Asset, rank: Int) {
    val verification = assetVerification(asset, rank) ?: return
    item {
        StatusItem(verification)
        Spacer16()
    }
}

@Composable
private fun StatusItem(verification: AssetVerification) {
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current

    PropertyItem(
        modifier = Modifier.clickable {
            uriHandler.open(
                context,
                AppUrl.docs(DocsUrl.TokenVerification)
            )
        },
        title = {
            PropertyTitleText(
                text = R.string.transaction_status,
                info = verification.infoSheetEntity()
            )
        },
        data = {
            PropertyDataText(
                stringResource(verification.labelRes()),
                color = verification.color(),
                badge = {
                    DataBadgeChevron(verification.badgeIconRes())
                }
            )
        },
        listPosition = ListPosition.Single,
    )
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
