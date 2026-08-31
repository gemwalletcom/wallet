package com.gemwallet.android.application.transactions.cases

import kotlinx.coroutines.flow.Flow

interface GetPendingTransactionsCount {

    fun getPendingTransactionsCount(): Flow<Int?>
}