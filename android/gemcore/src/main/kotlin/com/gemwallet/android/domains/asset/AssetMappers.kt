package com.gemwallet.android.domains.asset

import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.UTXO

fun NFTAsset.toGem(): String = toJson()

fun UTXO.toGem(): String = toJson()

fun List<UTXO>.toGem() = map { it.toGem() }
