package com.gemwallet.android.data.coordinators.wallet

import android.util.Log
import com.gemwallet.android.application.wallet.cases.DeleteWallet
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemWalletDeletion
import uniffi.gemstone.GemWalletService
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class DeleteWalletImpl @Inject constructor(
    private val walletService: GemWalletService,
    private val userConfig: UserConfig,
) : DeleteWallet {

    override suspend fun deleteWallet(
        walletId: WalletId,
        onBoard: () -> Unit,
        onComplete: () -> Unit
    ) = withContext(Dispatchers.IO) {
        val deletion = try {
            walletService.deleteWallet(walletId.id)
        } catch (error: Exception) {
            Log.e(TAG, "wallet removal failed for ${walletId.id}; retry delete to finish", error)
            return@withContext
        }

        val callback: () -> Unit = when (deletion) {
            GemWalletDeletion.WALLETS_REMAINING -> onComplete
            GemWalletDeletion.LAST_WALLET_DELETED -> {
                userConfig.reload()
                onBoard
            }
        }

        withContext(Dispatchers.Main) {
            callback()
        }
    }

    private companion object {
        const val TAG = "DeleteWallet"
    }
}
