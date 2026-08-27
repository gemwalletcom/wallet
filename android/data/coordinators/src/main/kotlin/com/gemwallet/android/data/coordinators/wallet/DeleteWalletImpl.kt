package com.gemwallet.android.data.coordinators.wallet

import android.util.Log
import com.gemwallet.android.application.wallet.coordinators.DeleteWallet
import com.gemwallet.android.blockchain.operators.DeleteKeyStoreOperator
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletType
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.withContext
import com.gemwallet.android.serializer.toJson
import uniffi.gemstone.GemWalletService
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class DeleteWalletImpl @Inject constructor(
    private val sessionRepository: SessionRepository,
    private val walletsRepository: WalletsRepository,
    private val deleteKeyStoreOperator: DeleteKeyStoreOperator,
    private val walletService: GemWalletService,
) : DeleteWallet {

    override suspend fun deleteWallet(
        walletId: WalletId,
        onBoard: () -> Unit,
        onComplete: () -> Unit
    ) = withContext(Dispatchers.IO) {
        val wallet = walletsRepository.getWallet(walletId).firstOrNull() ?: return@withContext

        // Delete the keystore before the DB row; if it fails, keep the wallet so the secret is never orphaned silently.
        if (wallet.type != WalletType.View && !deleteKeyStoreOperator(wallet)) {
            Log.e(TAG, "keystore delete failed for ${walletId.id}; keeping the wallet")
            return@withContext
        }
        val hasWallets = try {
            walletService.deleteWallet(wallet.toJson())
        } catch (error: Exception) {
            Log.e(TAG, "wallet removal failed for ${walletId.id}; retry delete to finish", error)
            return@withContext
        }


        val callback: () -> Unit = if (hasWallets) {
            onComplete
        } else {
            sessionRepository.reset()
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
