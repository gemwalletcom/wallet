package com.gemwallet.android.data.repositories.assets

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain

val tronUSDT = Asset(
    id = AssetId(Chain.Tron, "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"),
    name = "Tether",
    symbol = "USDT",
    decimals = 6,
    type = AssetType.TRC20,
)

val defaultTokenAssets = listOf(tronUSDT)

val visibleByDefault = listOf(
    AssetId(Chain.Bitcoin),
    AssetId(Chain.Ethereum),
    AssetId(Chain.SmartChain),
    AssetId(Chain.Solana),
    AssetId(Chain.Tron),
    tronUSDT.id,
)
