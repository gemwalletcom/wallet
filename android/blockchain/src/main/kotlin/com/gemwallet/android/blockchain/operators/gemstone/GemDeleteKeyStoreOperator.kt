package com.gemwallet.android.blockchain.operators.gemstone

import android.util.Log
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.blockchain.operators.DeleteKeyStoreOperator
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemKeystore
import uniffi.gemstone.keystoreIdForWallet
import java.io.File

class GemDeleteKeyStoreOperator(
    private val baseDir: String,
    private val passwordStore: PasswordStore,
) : DeleteKeyStoreOperator {

    override fun invoke(walletId: WalletId): Boolean {
        var deletedAll = true

        try {
            GemKeystore(baseDir).use { keystore -> keystore.delete(keystoreIdForWallet(walletId.id)) }
        } catch (e: Exception) {
            Log.e(TAG, "v4 keystore delete failed for ${walletId.id}", e)
            deletedAll = false
        }

        val legacyFile = File(baseDir, walletId.id)
        if (legacyFile.exists() && !legacyFile.delete()) {
            Log.e(TAG, "v3 keystore delete failed for ${walletId.id}")
            deletedAll = false
        }

        if (deletedAll) {
            passwordStore.removePassword(walletId.id)
        }
        return deletedAll
    }

    private companion object {
        const val TAG = "GemDeleteKeyStoreOperator"
    }
}
