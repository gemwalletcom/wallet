package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.application.session.coordinators.GetCurrentCurrency
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.gemstone.GemstoneWalletSessionStore
import com.gemwallet.android.data.repositories.gemstone.GemstoneWalletStore
import com.gemwallet.android.data.repositories.session.SessionRepositoryImpl
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.data.service.store.database.SessionDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemWalletSessionService
import javax.inject.Singleton
import uniffi.gemstone.GemPreferencesService

@InstallIn(SingletonComponent::class)
@Module
object SessionModule {
    @Singleton
    @Provides
    fun provideGemWalletSessionService(
        sessionDao: SessionDao,
        walletStore: GemstoneWalletStore,
    ): GemWalletSessionService = GemWalletSessionService(GemstoneWalletSessionStore(sessionDao), walletStore)

    @Singleton
    @Provides
    fun provideSessionRepository(
        sessionDao: SessionDao,
        walletsRepository: WalletsRepository,
        walletSessionService: GemWalletSessionService,
        preferencesService: GemPreferencesService,
    ): SessionRepository = SessionRepositoryImpl(
        sessionDao = sessionDao,
        walletsRepository = walletsRepository,
        walletSessionService = walletSessionService,
        preferencesService = preferencesService,
    )

    @Provides
    fun provideGetCurrentCurrency(sessionRepository: SessionRepository): GetCurrentCurrency = sessionRepository
}
