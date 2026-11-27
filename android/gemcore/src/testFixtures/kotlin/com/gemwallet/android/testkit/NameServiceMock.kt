package com.gemwallet.android.testkit

import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemNameRecordState
import uniffi.gemstone.GemNameServiceInterface

class NameServiceMock : GemNameServiceInterface {
    val requests = mutableListOf<Pair<String, Chain>>()

    override suspend fun getNameRecord(name: String, chain: uniffi.gemstone.Chain): GemNameRecordState {
        val requested = Chain.entries.first { it.string == chain }
        requests.add(name to requested)
        return GemNameRecordState.Complete(mockNameRecord(name = name, chain = chain))
    }
}
