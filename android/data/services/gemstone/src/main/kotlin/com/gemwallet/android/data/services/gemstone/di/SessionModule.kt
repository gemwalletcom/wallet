package com.gemwallet.android.data.services.gemstone.di

import com.gemwallet.android.data.services.gemstone.stores.GemstonePreferencesStore
import com.gemwallet.android.data.services.gemstone.stores.GemstoneWalletSessionStore
import com.gemwallet.android.data.services.gemstone.stores.GemstoneWalletStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemWalletSessionService
import uniffi.gemstone.GemWalletSessionServiceInterface
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object SessionModule {
    @Singleton
    @Provides
    fun provideGemstoneWalletSessionStore(
        preferences: GemstonePreferencesStore,
    ): GemstoneWalletSessionStore = GemstoneWalletSessionStore(preferences)

    @Singleton
    @Provides
    fun provideGemWalletSessionService(
        sessionStore: GemstoneWalletSessionStore,
        walletStore: GemstoneWalletStore,
    ): GemWalletSessionService = GemWalletSessionService(sessionStore, walletStore)

    @Singleton
    @Provides
    fun provideGemWalletSessionServiceInterface(service: GemWalletSessionService): GemWalletSessionServiceInterface = service
}
