package com.gemwallet.android.application.transactions.cases

interface SyncTransactions {
    suspend fun syncTransactions(): Boolean
}
