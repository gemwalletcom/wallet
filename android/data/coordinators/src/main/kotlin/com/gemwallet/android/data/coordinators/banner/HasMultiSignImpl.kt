package com.gemwallet.android.data.coordinators.banner

import com.gemwallet.android.cases.banners.HasMultiSign
import com.gemwallet.android.data.service.store.database.BannersDao
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.mapLatest

@OptIn(ExperimentalCoroutinesApi::class)
class HasMultiSignImpl(
    private val bannersDao: BannersDao,
) : HasMultiSign {

    override fun hasMultiSign(wallet: Wallet): Flow<Boolean> =
        bannersDao.getMultisign(wallet.id.id).mapLatest { it.isNotEmpty() }
}
