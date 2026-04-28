package com.gemwallet.android.data.service.store.database

import androidx.room.withTransaction
import javax.inject.Inject

interface StoreTransactionRunner {
    suspend fun <T> run(block: suspend () -> T): T
}

class RoomStoreTransactionRunner @Inject constructor(
    private val database: GemDatabase,
) : StoreTransactionRunner {
    override suspend fun <T> run(block: suspend () -> T): T {
        return database.withTransaction(block)
    }
}
