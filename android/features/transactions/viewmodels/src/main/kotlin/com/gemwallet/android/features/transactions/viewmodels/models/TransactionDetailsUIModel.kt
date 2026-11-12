package com.gemwallet.android.features.transactions.viewmodels.models

import androidx.compose.runtime.Immutable
import uniffi.gemstone.GemTransactionDetailRows
import uniffi.gemstone.GemTransactionDetailSection

@Immutable
data class TransactionDetailsUIModel(val rows: GemTransactionDetailRows, val sections: List<GemTransactionDetailSection>, val headerTarget: TransactionHeaderTarget?)
