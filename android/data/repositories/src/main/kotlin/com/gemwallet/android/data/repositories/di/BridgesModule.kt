package com.gemwallet.android.data.repositories.di

import android.content.Context
import com.gemwallet.android.data.repositories.bridge.BridgesRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.data.service.store.database.ConnectionsDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object BridgesModule {
    @Singleton
    @Provides
    fun provideBridgeRepository(
        @ApplicationContext context: Context,
        walletsRepository: WalletsRepository,
        connectionsDao: ConnectionsDao,
    ): BridgesRepository = BridgesRepository(
        context = context,
        walletsRepository = walletsRepository,
        connectionsDao = connectionsDao,
    )
}

