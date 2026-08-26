package com.gemwallet.android.services

import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.ScanTransactionPayload
import uniffi.gemstone.GemConfirmScanner
import uniffi.gemstone.ScanTransaction as GemScanTransaction
import uniffi.gemstone.ScanTransactionPayload as GemScanTransactionPayload

class DeviceConfirmScanner(
    private val apiClient: GemDeviceApiClient,
) : GemConfirmScanner {

    override suspend fun scanTransaction(payload: GemScanTransactionPayload): GemScanTransaction? {
        val request = runCatching { payload.decodeJson<ScanTransactionPayload>() }.getOrNull() ?: return null
        return runCatching { apiClient.getScanTransaction(request) }.getOrNull()?.toJson()
    }
}
