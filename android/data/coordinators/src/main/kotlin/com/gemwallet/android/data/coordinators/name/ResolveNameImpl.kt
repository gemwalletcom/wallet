package com.gemwallet.android.data.coordinators.name

import com.gemwallet.android.cases.name.ResolveName
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NameRecord
import uniffi.gemstone.GemNameService
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson

class ResolveNameImpl(
    private val nameService: GemNameService,
) : ResolveName {

    override suspend fun resolveName(name: String, chain: Chain): NameRecord? {
        return nameService.resolve(name, chain.string)?.decodeJson<NameRecord>()
    }
}
