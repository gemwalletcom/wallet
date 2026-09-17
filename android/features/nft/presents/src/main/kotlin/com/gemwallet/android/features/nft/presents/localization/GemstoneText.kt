package com.gemwallet.android.features.nft.presents.localization

import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemNftList

@StringRes
internal fun GemNftList.stringRes(): Int = when (this) {
    GemNftList.COLLECTIONS,
    GemNftList.COLLECTION -> R.string.nft_collections
    GemNftList.UNVERIFIED -> R.string.asset_verification_unverified
}
