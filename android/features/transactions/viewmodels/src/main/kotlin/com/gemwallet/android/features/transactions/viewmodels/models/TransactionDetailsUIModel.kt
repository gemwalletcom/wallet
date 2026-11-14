package com.gemwallet.android.features.transactions.viewmodels.models

import androidx.compose.runtime.Immutable
import uniffi.gemstone.GemGroup
import uniffi.gemstone.GemTransactionDetailRows

@Immutable
data class TransactionDetailsUIModel(val rows: GemTransactionDetailRows, val sections: List<GemGroup>, val headerTarget: TransactionHeaderTarget?)
