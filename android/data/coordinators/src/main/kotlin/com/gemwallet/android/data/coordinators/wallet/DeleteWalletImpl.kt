package com.gemwallet.android.data.coordinators.wallet

import android.util.Log
import com.gemwallet.android.application.wallet.cases.DeleteWallet
import com.gemwallet.android.blockchain.operators.DeleteKeyStoreOperator
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.application.session.cases.ClearSession
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemWalletService
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class DeleteWalletImpl @Inject constructor(
    private val clearSession: ClearSession,
    private val deleteKeyStoreOperator: DeleteKeyStoreOperator,
    private val walletService: GemWalletService,
    private val userConfig: UserConfig,
) : DeleteWallet {

    override suspend fun deleteWallet(
        walletId: WalletId,
        onBoard: () -> Unit,
        onComplete: () -> Unit
    ) = withContext(Dispatchers.IO) {
        if (!deleteKeyStoreOperator(walletId)) {
            Log.e(TAG, "keystore delete failed for ${walletId.id}; keeping the wallet")
            return@withContext
        }
        val hasWallets = try {
            walletService.deleteWallet(walletId.id)
        } catch (error: Exception) {
            Log.e(TAG, "wallet removal failed for ${walletId.id}; retry delete to finish", error)
            return@withContext
        }

        val callback: () -> Unit = if (hasWallets) {
            onComplete
        } else {
            clearSession.clearSession()
            userConfig.reload()
            onBoard
        }

        withContext(Dispatchers.Main) {
            callback()
        }
    }

    private companion object {
        const val TAG = "DeleteWallet"
    }
}
