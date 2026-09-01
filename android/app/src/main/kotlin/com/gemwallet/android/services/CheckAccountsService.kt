package com.gemwallet.android.services

import android.util.Log
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemAppStartService
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class CheckAccountsService @Inject constructor(
    private val appStartService: GemAppStartService,
) {
    suspend operator fun invoke() = withContext(Dispatchers.IO) {
        appStartService.setupWallets().forEach { failure ->
            Log.e("CheckAccountsService", "${failure.step} failed: ${failure.message}")
        }
    }
}
