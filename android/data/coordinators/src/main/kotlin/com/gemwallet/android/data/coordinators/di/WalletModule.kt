package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.wallet.cases.GetAllWallets
import com.gemwallet.android.data.coordinators.wallet.GetAllWalletsImpl
import com.gemwallet.android.data.services.store.queries.WalletsQuery
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object WalletModule {

    @Provides
    @Singleton
    fun provideGetAllWallets(getSession: GetSession, walletsQuery: WalletsQuery): GetAllWallets = GetAllWalletsImpl(getSession, walletsQuery)
}
