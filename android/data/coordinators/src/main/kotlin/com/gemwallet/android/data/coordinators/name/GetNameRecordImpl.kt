package com.gemwallet.android.data.coordinators.name

import com.gemwallet.android.application.recipient.cases.GetNameRecord
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NameRecord
import uniffi.gemstone.GemNameService
import com.gemwallet.android.serializer.decodeJson

class GetNameRecordImpl(
    private val nameService: GemNameService,
) : GetNameRecord {

    override suspend fun getNameRecord(name: String, chain: Chain): NameRecord? {
        return nameService.getNameRecord(name, chain.string)?.decodeJson<NameRecord>()
    }

    override fun isNameSupported(name: String): Boolean = nameService.isNameSupported(name)
}
