package com.gemwallet.android.data.coordinators.recipient

import com.gemwallet.android.application.recipient.coordinators.GetWallets
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.flow.Flow

class GetWalletsImpl(
    private val walletsRepository: WalletsRepository,
) : GetWallets {
    override fun getAll(): Flow<List<Wallet>> = walletsRepository.getAll()
}
