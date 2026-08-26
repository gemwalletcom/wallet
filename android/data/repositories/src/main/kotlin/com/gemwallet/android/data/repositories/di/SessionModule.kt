package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.application.session.coordinators.GetCurrentCurrency
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.gemstone.GemstoneWalletSessionStore
import com.gemwallet.android.data.repositories.gemstone.GemstoneWalletStore
import com.gemwallet.android.data.repositories.session.SessionRepositoryImpl
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.data.service.store.database.SessionDao
import dagger.Lazy
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemWalletSessionService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object SessionModule {
    @Singleton
    @Provides
    fun provideGemWalletSessionService(
        sessionDao: SessionDao,
        walletsRepository: Lazy<WalletsRepository>,
    ): GemWalletSessionService = GemWalletSessionService(GemstoneWalletSessionStore(sessionDao), GemstoneWalletStore(walletsRepository))

    @Singleton
    @Provides
    fun provideSessionRepository(
        sessionDao: SessionDao,
        walletsRepository: WalletsRepository,
        walletSessionService: GemWalletSessionService,
    ): SessionRepository = SessionRepositoryImpl(
        sessionDao = sessionDao,
        walletsRepository = walletsRepository,
        walletSessionService = walletSessionService,
    )

    @Provides
    fun provideGetCurrentCurrency(sessionRepository: SessionRepository): GetCurrentCurrency = sessionRepository
}
