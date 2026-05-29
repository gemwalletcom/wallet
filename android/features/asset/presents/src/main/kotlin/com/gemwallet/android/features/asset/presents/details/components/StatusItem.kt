package com.gemwallet.android.features.asset.presents.details.components

import androidx.compose.foundation.lazy.LazyListScope
import com.gemwallet.android.ext.type
import com.gemwallet.android.ui.components.list_item.property.verificationStatusItem
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetSubtype
import com.wallet.core.primitives.VerificationStatus

internal fun LazyListScope.status(asset: Asset, rank: Int) {
    val status = assetVerification(asset, rank) ?: return
    verificationStatusItem(status)
}

internal fun assetVerification(asset: Asset, rank: Int): VerificationStatus? {
    if (asset.id.type() == AssetSubtype.NATIVE) {
        return null
    }
    return rank.verificationStatus()
}

private const val SUSPICIOUS_MAX_SCORE = 5
private const val UNVERIFIED_MAX_SCORE = 15

private fun Int.verificationStatus(): VerificationStatus? = when {
    this <= SUSPICIOUS_MAX_SCORE -> VerificationStatus.Suspicious
    this <= UNVERIFIED_MAX_SCORE -> VerificationStatus.Unverified
    else -> null
}
