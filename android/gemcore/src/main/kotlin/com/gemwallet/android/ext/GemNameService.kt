package com.gemwallet.android.ext

import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemNameRecordState
import uniffi.gemstone.GemNameServiceInterface

suspend fun GemNameServiceInterface.getNameRecord(name: String, chain: Chain): GemNameRecordState = getNameRecord(name, chain.string)
