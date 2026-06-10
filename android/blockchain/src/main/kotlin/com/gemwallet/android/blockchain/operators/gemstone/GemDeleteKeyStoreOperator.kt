package com.gemwallet.android.blockchain.operators.gemstone

import android.util.Log
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.blockchain.operators.DeleteKeyStoreOperator
import com.gemwallet.android.ext.keystoreId
import com.wallet.core.primitives.Wallet
import uniffi.gemstone.GemKeystore
import java.io.File

class GemDeleteKeyStoreOperator(
    private val baseDir: String,
    private val passwordStore: PasswordStore,
) : DeleteKeyStoreOperator {

    override fun invoke(wallet: Wallet): Boolean {
        var deletedAll = true

        try {
            GemKeystore(baseDir).use { keystore -> keystore.delete(wallet.keystoreId) }
        } catch (e: Exception) {
            Log.e(TAG, "v4 keystore delete failed for ${wallet.id.id}", e)
            deletedAll = false
        }

        // Remove the v3 file left in place for downgrade safety (or one that never migrated).
        val legacyFile = File(baseDir, wallet.id.id)
        if (legacyFile.exists() && !legacyFile.delete()) {
            Log.e(TAG, "v3 keystore delete failed for ${wallet.id.id}")
            deletedAll = false
        }

        if (deletedAll) {
            passwordStore.removePassword(wallet.id.id)
        }
        return deletedAll
    }

    private companion object {
        const val TAG = "GemDeleteKeyStoreOperator"
    }
}
