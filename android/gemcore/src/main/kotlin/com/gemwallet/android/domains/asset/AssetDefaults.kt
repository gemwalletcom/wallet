package com.gemwallet.android.domains.asset

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetBasic
import com.wallet.core.primitives.Chain


val Chain.defaultAssets: List<Asset>
    get() = assetConfig.walletDefaultAssets(string).map { it.toPrimitives() }

val Asset.defaultBasic: AssetBasic
    get() = assetConfig.defaultAssetBasic(toGem()).decodeJson()
