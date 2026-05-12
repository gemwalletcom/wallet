package com.gemwallet.android.data.coordinators.wallet

import androidx.compose.runtime.Stable
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.wallet.coordinators.GetWalletSecretData
import com.gemwallet.android.blockchain.operators.LoadPrivateDataOperator
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.domains.wallet.values.WalletSecretDataValue
import com.wallet.core.primitives.WalletId
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
            val data = try {
                wallet?.let {
                    val password = passwordStore.getPassword(wallet.id.id)
                    val phrase = loadPrivateDataOperator(wallet, password)
                    phrase
                }
            } catch (_: Throwable) {
                null
            }?.split(" ") ?: emptyList()
            WalletSecretDataValueImpl(data)
        }
    }
}

@Stable
class WalletSecretDataValueImpl(override val data: List<String>) : WalletSecretDataValue {
    override fun phrase(): List<String> {
        return data.takeIf { data.size >= 12 } ?: emptyList()
    }

    override fun privateKey(): String? {
        return data.takeIf { data.size == 1 }?.firstOrNull()
    }

    override fun toString(): String = data.joinToString(" ")

}
