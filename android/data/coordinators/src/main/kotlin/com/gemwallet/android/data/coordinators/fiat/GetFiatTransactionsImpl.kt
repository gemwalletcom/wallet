package com.gemwallet.android.data.coordinators.fiat

import com.gemwallet.android.application.fiat.coordinators.GetFiatTransactions
import com.wallet.core.primitives.FiatTransactionData
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemFiatService
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson

class GetFiatTransactionsImpl(
    private val fiatService: GemFiatService,
) : GetFiatTransactions {
    override suspend fun invoke(walletId: WalletId): List<FiatTransactionData> {
        return fiatService.getTransactions(walletId.id).map { it.decodeJson<FiatTransactionData>() }
    }
}
