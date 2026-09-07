package com.gemwallet.android.ext

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NameRecord
import uniffi.gemstone.GemNameServiceInterface
import uniffi.gemstone.GemRecipientValidation

fun GemNameServiceInterface.validateRecipient(chain: Chain, input: String, nameRecord: NameRecord?): GemRecipientValidation =
    validateRecipient(chain.string, input, nameRecord?.toGem())

suspend fun GemNameServiceInterface.getNameRecord(name: String, chain: Chain): NameRecord? =
    getNameRecord(name, chain.string)?.toPrimitives()
