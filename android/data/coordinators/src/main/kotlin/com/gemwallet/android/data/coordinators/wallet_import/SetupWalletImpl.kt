package com.gemwallet.android.data.coordinators.wallet_import

import android.util.Log
import com.gemwallet.android.application.wallet_import.cases.SetupWallet
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Wallet
import uniffi.gemstone.GemAppStartService

class SetupWalletImpl(
    private val appStartService: GemAppStartService,
) : SetupWallet {
    override suspend fun setup(wallet: Wallet) {
        appStartService.setupWallet(wallet.toJson()).forEach { failure ->
            Log.e("SetupWallet", "${failure.step} failed: ${failure.message}")
        }
    }
}
