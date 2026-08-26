package com.gemwallet.android.domains.asset

import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.UTXO
import uniffi.gemstone.Asset as GemAsset

fun Asset.toGem(): GemAsset = toJson()

fun GemAsset.toPrimitives(): Asset? = runCatching { decodeJson<Asset>() }.getOrNull()

fun GemAsset.toDTO(): Asset = decodeJson()

fun NFTAsset.toGem(): String = toJson()

fun UTXO.toGem(): String = toJson()

fun List<UTXO>.toGem() = map { it.toGem() }
