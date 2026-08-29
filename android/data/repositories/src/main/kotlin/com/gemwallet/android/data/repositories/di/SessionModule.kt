package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.application.session.cases.GetCurrentCurrency
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
    fun provideGemstoneWalletSessionStore(
        sessionDao: SessionDao,
        preferencesService: GemPreferencesService,
    ): GemstoneWalletSessionStore = GemstoneWalletSessionStore(sessionDao, preferencesService)

    @Singleton
    @Provides
    fun provideGemWalletSessionService(
        sessionStore: GemstoneWalletSessionStore,
        walletStore: GemstoneWalletStore,
    ): GemWalletSessionService = GemWalletSessionService(sessionStore, walletStore)

    @Singleton
    @Provides
    fun provideSessionRepository(
        sessionStore: GemstoneWalletSessionStore,
        walletStore: GemstoneWalletStore,
        walletSessionService: GemWalletSessionService,
        preferencesService: GemPreferencesService,
    ): SessionRepository = SessionRepositoryImpl(
        sessionStore = sessionStore,
        walletStore = walletStore,
        walletSessionService = walletSessionService,
        preferencesService = preferencesService,
    )

    @Provides
    fun provideGetCurrentCurrency(sessionRepository: SessionRepository): GetCurrentCurrency = sessionRepository
}
