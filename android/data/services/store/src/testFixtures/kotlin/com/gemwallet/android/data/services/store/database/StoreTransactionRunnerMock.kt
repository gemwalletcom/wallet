package com.gemwallet.android.data.services.store.database

fun mockStoreTransactionRunner(): StoreTransactionRunner = object : StoreTransactionRunner {
    override suspend fun <T> run(block: suspend () -> T): T = block()
}
