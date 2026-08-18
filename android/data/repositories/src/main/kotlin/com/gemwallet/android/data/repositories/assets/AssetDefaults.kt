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

val tempoUSDC = Asset(
    id = AssetId(Chain.Tempo, "0x20C000000000000000000000b9537d11c60E8b50"),
    name = "Bridged USDC",
    symbol = "USDC.e",
    decimals = 6,
    type = AssetType.TIP20,
)

val tempoPathUSD = Asset(
    id = AssetId(Chain.Tempo, "0x20C0000000000000000000000000000000000000"),
    name = "pathUSD",
    symbol = "pathUSD",
    decimals = 6,
    type = AssetType.TIP20,
)

val defaultTokenAssets = listOf(tronUSDT, tempoUSDC, tempoPathUSD)

val visibleByDefault = listOf(
    AssetId(Chain.Bitcoin),
    AssetId(Chain.Ethereum),
    AssetId(Chain.SmartChain),
    AssetId(Chain.Solana),
    AssetId(Chain.Tron),
    tronUSDT.id,
    tempoUSDC.id,
)
