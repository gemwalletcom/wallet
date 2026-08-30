package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.application.session.cases.GetCurrentWallet
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.session.cases.SetCurrentCurrency
import com.gemwallet.android.application.wallet.cases.SetCurrentWallet
import com.gemwallet.android.data.coordinators.session.GetCurrentWalletIdImpl
import com.gemwallet.android.data.coordinators.session.SessionCoordinator
import com.gemwallet.android.data.services.gemstone.stores.GemstoneWalletSessionStore
import com.gemwallet.android.data.services.gemstone.stores.GemstoneWalletStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemDeviceService
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemPriceService
import uniffi.gemstone.GemWalletSessionService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object SessionModule {

    @Provides
    @Singleton
    fun provideSessionCoordinator(
        sessionStore: GemstoneWalletSessionStore,
        walletStore: GemstoneWalletStore,
        walletSessionService: GemWalletSessionService,
        preferencesService: GemPreferencesService,
        priceService: GemPriceService,
        deviceService: GemDeviceService,
    ): SessionCoordinator = SessionCoordinator(
        sessionStore = sessionStore,
        walletStore = walletStore,
        walletSessionService = walletSessionService,
        preferencesService = preferencesService,
        priceService = priceService,
        deviceService = deviceService,
    )

    @Provides
    @Singleton
    fun provideGetSession(coordinator: SessionCoordinator): GetSession = coordinator

    @Provides
    @Singleton
    fun provideGetCurrentWallet(coordinator: SessionCoordinator): GetCurrentWallet = coordinator

    @Provides
    @Singleton
    fun provideGetCurrentCurrency(coordinator: SessionCoordinator): GetCurrentCurrency = coordinator

    @Provides
    @Singleton
    fun provideSetCurrentCurrency(coordinator: SessionCoordinator): SetCurrentCurrency = coordinator

    @Provides
    @Singleton
    fun provideSetCurrentWallet(coordinator: SessionCoordinator): SetCurrentWallet = coordinator

    @Provides
    @Singleton
    fun provideGetCurrentWalletId(getSession: GetSession): GetCurrentWalletId = GetCurrentWalletIdImpl(getSession)
}
