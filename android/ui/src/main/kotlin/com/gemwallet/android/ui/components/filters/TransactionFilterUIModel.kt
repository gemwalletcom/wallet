package com.gemwallet.android.ui.components.filters

import android.content.Context
import com.gemwallet.android.ext.GemConstants
import com.gemwallet.android.ui.localization.getLabel
import uniffi.gemstone.GemTransactionFilter

data class TransactionFilterUIModel(val filter: GemTransactionFilter, val title: String)

fun transactionFilterOptions(context: Context): List<TransactionFilterUIModel> = GemConstants.transactionFilters.map { TransactionFilterUIModel(it, context.getString(it.getLabel())) }
