package com.gemwallet.android.domains.asset

import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.UTXO


fun UTXO.toGem(): String = toJson()

fun List<UTXO>.toGem() = map { it.toGem() }
