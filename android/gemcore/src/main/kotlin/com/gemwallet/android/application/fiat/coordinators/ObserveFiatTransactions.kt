package com.gemwallet.android.application.fiat.coordinators

import com.wallet.core.primitives.FiatTransactionAssetData
import kotlinx.coroutines.flow.Flow

interface ObserveFiatTransactions {
    operator fun invoke(): Flow<List<FiatTransactionAssetData>>
}
