package com.gemwallet.android.data.coordinators.wallet

import com.gemwallet.android.application.wallet.cases.DeleteWallet
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemWalletDeletion
import uniffi.gemstone.GemWalletService
import uniffi.gemstone.GemWalletServiceInterface
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class DeleteWalletImpl @Inject constructor(private val walletService: GemWalletServiceInterface, private val userConfig: UserConfig) : DeleteWallet {

    override suspend fun deleteWallet(walletId: WalletId, onBoard: () -> Unit, onComplete: () -> Unit) = withContext(Dispatchers.IO) {
        val deletion = walletService.deleteWallet(walletId.id)

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
}
