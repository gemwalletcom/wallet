package com.gemwallet.android.domains.asset

import com.gemwallet.android.ext.isStakeSupported
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.type
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetBasic
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetProperties
import com.wallet.core.primitives.AssetScore
import com.wallet.core.primitives.AssetSubtype
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemAssetConfigService

private val assetConfig = GemAssetConfigService()

val Chain.defaultAssets: List<Asset>
    get() = assetConfig.walletDefaultAssets(string).map { it.toDTO() }

val Asset.defaultBasic: AssetBasic
    get() {
        val isNative = id.type() == AssetSubtype.NATIVE
        val score = id.defaultScore
        val isEnabled = score.rank >= 0
        return AssetBasic(
            asset = this,
            properties = AssetProperties(
                isEnabled = isEnabled,
                isBuyable = false,
                isSellable = false,
                isSwapable = assetConfig.isSwapable(id.toIdentifier()),
                isStakeable = isEnabled && isNative && id.chain.isStakeSupported(),
                isEarnable = false,
                hasImage = false,
            ),
            score = score,
        )
    }

private val AssetId.defaultScore: AssetScore
    get() = AssetScore(rank = assetConfig.defaultRank(toIdentifier()))
