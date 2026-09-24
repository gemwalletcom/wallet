package com.gemwallet.android.domains.asset

import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.byChain
import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.StakeChain
import uniffi.gemstone.GemAssetText
import uniffi.gemstone.assetText

val Asset.chain: Chain
    get() = id.chain

val Asset.stakeChain: StakeChain?
    get() = StakeChain.byChain(id.chain)

private val Asset.text: GemAssetText
    get() = assetText(toGem())

val Asset.networkFullName: String
    get() = text.networkFullName

val Asset.subtitleSymbol: String?
    get() = text.subtitleSymbol
