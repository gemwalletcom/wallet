package com.gemwallet.android.data.coordinators.name

import com.gemwallet.android.cases.name.ResolveName
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NameRecord

class ResolveNameImpl(
    private val gemDeviceApiClient: GemDeviceApiClient,
) : ResolveName {

    override suspend fun resolveName(name: String, chain: Chain): NameRecord? {
        return gemDeviceApiClient.resolve(name, chain.string)
    }
}
