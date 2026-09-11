package com.gemwallet.android.ext

import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemNameRecordState
import uniffi.gemstone.GemNameServiceInterface
import uniffi.gemstone.GemRecipientValidation

fun GemNameServiceInterface.validateRecipient(chain: Chain, input: String, state: GemNameRecordState): GemRecipientValidation =
    validateRecipient(chain.string, input, state)

suspend fun GemNameServiceInterface.getNameRecord(name: String, chain: Chain): GemNameRecordState =
    getNameRecord(name, chain.string)
