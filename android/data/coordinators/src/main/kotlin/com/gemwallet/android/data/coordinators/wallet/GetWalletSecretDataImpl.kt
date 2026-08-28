package com.gemwallet.android.data.coordinators.wallet

import android.util.Log
import androidx.compose.runtime.Stable
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.wallet.coordinators.GetWalletSecretData
import com.gemwallet.android.blockchain.operators.LoadPrivateDataOperator
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.domains.wallet.values.WalletSecretDataValue
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletType
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.mapLatest

@OptIn(ExperimentalCoroutinesApi::class)
class GetWalletSecretDataImpl(
    private val walletsRepository: WalletsRepository,
    private val passwordStore: PasswordStore,
    private val loadPrivateDataOperator: LoadPrivateDataOperator,
) : GetWalletSecretData {

    override fun getSecretData(walletId: WalletId): Flow<WalletSecretDataValue> {
        return walletsRepository.getWallet(walletId).mapLatest { wallet ->
            wallet ?: return@mapLatest WalletSecretDataValueImpl(emptyList())
            try {
                val password = passwordStore.getPassword(wallet.id.id)
                val secret = loadPrivateDataOperator(wallet, password)
                when (wallet.type) {
                    WalletType.PrivateKey -> WalletSecretDataValueImpl(listOf(secret), isPrivateKey = true)
                    else -> WalletSecretDataValueImpl(secret.split(" "))
                }
            } catch (e: CancellationException) {
                throw e
            } catch (error: Throwable) {
                Log.e(TAG, "secret data unavailable for ${walletId.id}", error)
                WalletSecretDataValueImpl(emptyList(), isError = true)
            }
        }
    }

    private companion object {
        const val TAG = "GetWalletSecretData"
    }
}

@Stable
class WalletSecretDataValueImpl(
    override val data: List<String>,
    override val isError: Boolean = false,
    private val isPrivateKey: Boolean = false,
) : WalletSecretDataValue {
    override fun phrase(): List<String> = if (isPrivateKey) emptyList() else data

    override fun privateKey(): String? = data.firstOrNull().takeIf { isPrivateKey }

    override fun toString(): String = data.joinToString(" ")
}
