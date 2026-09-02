package com.gemwallet.android.domains.name

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NameRecord
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemRecipientValidation

interface AddressInputResolving {
    fun validateRecipient(chain: Chain, input: String, nameRecord: NameRecord?): GemRecipientValidation

    fun recipient(chain: Chain, input: String, nameRecord: NameRecord?, memo: String?, references: List<String>): GemRecipient

    fun isNameSupported(name: String): Boolean

    suspend fun getNameRecord(name: String, chain: Chain): NameRecord?
}
