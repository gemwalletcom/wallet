package com.gemwallet.android.application.security.cases

interface KeystoreRecovery {
    suspend fun repairLockStores(): Boolean

    suspend fun resetLocalData()
}
