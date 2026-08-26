package com.gemwallet.android.domains.asset

import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.UTXO
import uniffi.gemstone.GemAsset

fun Asset.toGem() = GemAsset(
    id = id.toIdentifier(),
    chain = id.chain.string,
    tokenId = id.tokenId,
    name = name,
    symbol = symbol,
    decimals = decimals,
    assetType = type.toJson(),
)

fun GemAsset.toPrimitives(): Asset? {
    val assetId = id.toAssetId() ?: return null
    return Asset(
        id = assetId,
        name = name,
        symbol = symbol,
        decimals = decimals,
        type = assetType.decodeJson(),
    )
}

fun NFTAsset.toGem(): String = toJson()

fun GemAsset.toDTO() = Asset(
    id = AssetId(id),
    name = name,
    symbol = symbol,
    decimals = decimals,
    type = assetType.decodeJson(),
)

fun UTXO.toGem(): String = toJson()

fun List<UTXO>.toGem() = map { it.toGem() }
