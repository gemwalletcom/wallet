package com.gemwallet.android.data.coordinators.fiat

import com.gemwallet.android.application.fiat.coordinators.GetFiatTransactions
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.wallet.core.primitives.FiatTransactionInfo

class GetFiatTransactionsImpl(
    private val gemDeviceApiClient: GemDeviceApiClient,
) : GetFiatTransactions {
    override suspend fun getFiatTransactions(walletId: String): List<FiatTransactionInfo> {
        return gemDeviceApiClient.getFiatTransactions(walletId)
    }
}
