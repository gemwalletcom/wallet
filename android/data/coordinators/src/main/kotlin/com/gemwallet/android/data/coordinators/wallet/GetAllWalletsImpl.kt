package com.gemwallet.android.data.coordinators.wallet

import androidx.compose.runtime.Stable
import com.gemwallet.android.application.wallet.cases.GetAllWallets
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.domains.wallet.aggregates.WalletDataAggregate
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletType
import uniffi.gemstone.GemWalletRulesService
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest

private val walletRules = GemWalletRulesService()

@OptIn(ExperimentalCoroutinesApi::class)
class GetAllWalletsImpl(
    private val sessionRepository: SessionRepository,
    private val walletsRepository: WalletsRepository,
) : GetAllWallets {

    override fun getAllWallets(): Flow<List<WalletDataAggregate>> {
        return sessionRepository.session().flatMapLatest { session ->
            val currentWalletId = session?.wallet?.id
            walletsRepository.getAll().map { items ->
                walletRules.sortedWallets(items.map { it.toJson() }).map { it.decodeJson<Wallet>() }
            }.mapLatest { items ->
                items.map { WalletDataAggregateImpl(it, it.id == currentWalletId) }
            }
        }
    }
}

@Stable
class WalletDataAggregateImpl(
    private val wallet: Wallet,
    override val isCurrent: Boolean,
) : WalletDataAggregate {

    override val id: String = wallet.id.id

    override val name: String = wallet.name

    override val type: WalletType = wallet.type

    override val walletChain: Chain? = walletAccount?.chain

    override val walletAddress: String? = walletAccount?.address

    override val isPinned: Boolean = wallet.isPinned

    override val imageUrl: String? = wallet.imageUrl

    private val walletAccount: Account?
        get() = walletRules.displayAccount(wallet.toJson())?.toPrimitives()
}
