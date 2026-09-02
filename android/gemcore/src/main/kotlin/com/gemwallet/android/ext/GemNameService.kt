package com.gemwallet.android.ext

import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NameRecord
import uniffi.gemstone.GemNameServiceInterface
import uniffi.gemstone.GemRecipientValidation

fun GemNameServiceInterface.validateRecipient(chain: Chain, input: String, nameRecord: NameRecord?): GemRecipientValidation =
    validateRecipient(chain.string, input, nameRecord?.toJson())

suspend fun GemNameServiceInterface.getNameRecord(name: String, chain: Chain): NameRecord? =
    getNameRecord(name, chain.string)?.decodeJson<NameRecord>()
