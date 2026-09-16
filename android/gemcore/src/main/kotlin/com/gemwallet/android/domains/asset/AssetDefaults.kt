package com.gemwallet.android.domains.asset

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetBasic


val Asset.defaultBasic: AssetBasic
    get() = assetConfig.defaultAssetBasic(toGem()).toPrimitives()
