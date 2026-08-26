package com.gemwallet.android.services

import com.gemwallet.android.blockchain.gemstone.toGem
import com.gemwallet.android.blockchain.gemstone.toPrimitives
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import uniffi.gemstone.GemConfirmScanner
import uniffi.gemstone.ScanTransaction as GemScanTransaction
import uniffi.gemstone.ScanTransactionPayload as GemScanTransactionPayload

class DeviceConfirmScanner(
    private val apiClient: GemDeviceApiClient,
) : GemConfirmScanner {

    override suspend fun scanTransaction(payload: GemScanTransactionPayload): GemScanTransaction? {
        val request = payload.toPrimitives() ?: return null
        return runCatching { apiClient.getScanTransaction(request) }.getOrNull()?.toGem()
    }
}
