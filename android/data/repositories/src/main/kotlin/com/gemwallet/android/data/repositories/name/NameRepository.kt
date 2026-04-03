package com.gemwallet.android.data.repositories.name

import android.util.Log
import com.gemwallet.android.cases.name.ResolveNameCase
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NameRecord
import java.util.Locale

class NameRepository(
    private val gemDeviceApiClient: GemDeviceApiClient,
) : ResolveNameCase {

    override suspend fun resolveName(name: String, chain: Chain): NameRecord? {
        return try {
            gemDeviceApiClient.resolve(name.lowercase(Locale.getDefault()), chain.string)
        } catch (err: Throwable) {
            Log.e("NameRepository", "Failed to resolve name: $name on ${chain.string}", err)
            null
        }
    }
}
