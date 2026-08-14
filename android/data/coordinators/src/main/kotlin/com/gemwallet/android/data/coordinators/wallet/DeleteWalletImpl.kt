package com.gemwallet.android.data.coordinators.wallet

import android.util.Log
import com.gemwallet.android.application.wallet.coordinators.DeleteWallet
import com.gemwallet.android.blockchain.operators.DeleteKeyStoreOperator
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.data.service.store.LocalStore
import com.gemwallet.android.data.service.store.WalletPreferencesFactory
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletType
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.withContext

class DeleteWalletImpl(
    private val sessionRepository: SessionRepository,
    private val walletsRepository: WalletsRepository,
    private val deleteKeyStoreOperator: DeleteKeyStoreOperator,
    private val walletPreferencesFactory: WalletPreferencesFactory,
    private val localStore: LocalStore,
) : DeleteWallet {

    override suspend fun deleteEmptyWallets(): Boolean = withContext(Dispatchers.IO) {
        for (wallet in walletsRepository.getEmptyWallets()) {
            if (!deleteWallet(wallet)) return@withContext false
        }
        true
    }

    override suspend fun deleteWallet(
        walletId: WalletId,
        onBoard: () -> Unit,
        onComplete: () -> Unit,
    ): Boolean = withContext(Dispatchers.IO) {
        val wallet = walletsRepository.getWallet(walletId).firstOrNull() ?: return@withContext false
        deleteWallet(wallet, onBoard, onComplete)
    }

    private suspend fun deleteWallet(
        wallet: Wallet,
        onBoard: () -> Unit = {},
        onComplete: () -> Unit = {},
    ): Boolean {
        val walletId = wallet.id
        val currentWalletId = sessionRepository.session().firstOrNull()?.wallet?.id

        // Delete the keystore before the DB row; if it fails, keep the wallet so the secret is never orphaned silently.
        if (wallet.type != WalletType.View && !deleteKeyStoreOperator(wallet)) {
            Log.e(TAG, "keystore delete failed for ${walletId.id}; keeping the wallet")
            return false
        }
        if (!walletsRepository.removeWallet(walletId = walletId)) {
            Log.e(TAG, "wallet row removal failed for ${walletId.id}; retry delete to finish")
            return false
        }

        walletPreferencesFactory.create(walletId.id).clear()
        if (!localStore.remove(wallet.imageUrl)) {
            Log.e(TAG, "wallet avatar delete failed for ${walletId.id}")
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
        return true
    }

    private companion object {
        const val TAG = "DeleteWallet"
    }
}
