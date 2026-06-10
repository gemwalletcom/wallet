package com.gemwallet.android.data.coordinators.wallet

import android.util.Log
import com.gemwallet.android.application.wallet.coordinators.DeleteWallet
import com.gemwallet.android.blockchain.operators.DeleteKeyStoreOperator
import com.gemwallet.android.cases.device.SyncSubscription
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletType
import kotlinx.coroutines.CoroutineExceptionHandler
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class DeleteWalletImpl @Inject constructor(
    private val sessionRepository: SessionRepository,
    private val walletsRepository: WalletsRepository,
    private val deleteKeyStoreOperator: DeleteKeyStoreOperator,
    private val syncSubscription: SyncSubscription,
) : DeleteWallet {

    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO + CoroutineExceptionHandler { _, _ -> })

    override suspend fun deleteWallet(
        walletId: WalletId,
        onBoard: () -> Unit,
        onComplete: () -> Unit
    ) = withContext(Dispatchers.IO) {
        val wallet = walletsRepository.getWallet(walletId).firstOrNull() ?: return@withContext
        val currentWalletId = sessionRepository.session().firstOrNull()?.wallet?.id

        // Delete the keystore before the DB row; if it fails, keep the wallet so the secret is never orphaned silently.
        if (wallet.type != WalletType.View && !deleteKeyStoreOperator(wallet)) {
            Log.e(TAG, "keystore delete failed for ${walletId.id}; keeping the wallet")
            return@withContext
        }
        if (!walletsRepository.removeWallet(walletId = walletId)) {
            Log.e(TAG, "wallet row removal failed for ${walletId.id}; retry delete to finish")
            return@withContext
        }

        scope.launch {
            walletsRepository.getAll().firstOrNull()
                ?.let { syncSubscription.syncSubscription(it) }
        }

        val callback: () -> Unit = if (currentWalletId == walletId) {
            val nextWallet = walletsRepository.getAll().firstOrNull()
                ?.filter { it.id != walletId }
                ?.sortedBy { it.type }
                ?.minByOrNull { it.index }

            if (nextWallet == null) {
                sessionRepository.reset()
                onBoard
            } else {
                sessionRepository.setWallet(nextWallet)
                onComplete
            }
        } else {
            onComplete
        }

        withContext(Dispatchers.Main) {
            callback()
        }
    }

    private companion object {
        const val TAG = "DeleteWallet"
    }
}
