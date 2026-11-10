package com.gemwallet.android.data.services.store.database

class StoreTransactionRunnerMock : StoreTransactionRunner {
    var runCount = 0
        private set

    override suspend fun <T> run(block: suspend () -> T): T {
        runCount += 1
        return block()
    }
}
