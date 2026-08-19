package com.gemwallet.android.domains.asset

import com.gemwallet.android.ext.isStakeSupported
import com.gemwallet.android.ext.isSwapSupport
import com.gemwallet.android.ext.type
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetBasic
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetProperties
import com.wallet.core.primitives.AssetScore
import com.wallet.core.primitives.AssetSubtype
import com.wallet.core.primitives.Chain
import uniffi.gemstone.assetDefaultRank
import uniffi.gemstone.defaultTokenRank

val Chain.defaultAssetRank: Int
    get() = assetDefaultRank(string)

val networkAnchorAssetIds = setOf(AssetId(Chain.Tempo))

val Asset.defaultBasic: AssetBasic
    get() = AssetBasic(
        asset = this,
        properties = id.defaultProperties,
        score = id.defaultScore,
    )

private val AssetId.defaultProperties: AssetProperties
    get() {
        val isNative = type() == AssetSubtype.NATIVE
        return AssetProperties(
            isEnabled = true,
            isBuyable = false,
            isSellable = false,
            isSwapable = this !in networkAnchorAssetIds && chain.isSwapSupport(),
            isStakeable = isNative && chain.isStakeSupported(),
            isEarnable = false,
            hasImage = false,
        )
    }

private val AssetId.defaultScore: AssetScore
    get() = AssetScore(
        rank = when (type()) {
            AssetSubtype.NATIVE -> chain.defaultAssetRank
            AssetSubtype.TOKEN -> defaultTokenRank()
        }
    )
