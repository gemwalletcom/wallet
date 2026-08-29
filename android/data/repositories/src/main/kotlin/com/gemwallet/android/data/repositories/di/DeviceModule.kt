package com.gemwallet.android.data.repositories.di

import android.content.Context
import com.gemwallet.android.cases.device.GetPushEnabled
import com.gemwallet.android.cases.device.GetPushToken
import com.gemwallet.android.cases.device.IsDeviceRegistered
import com.gemwallet.android.cases.device.SetPushToken
import com.gemwallet.android.cases.device.SwitchPushEnabled
import com.gemwallet.android.data.repositories.device.DeviceObserverService
import com.gemwallet.android.data.repositories.gemstone.GemstoneDevicePlatform
import com.gemwallet.android.data.repositories.gemstone.GemstoneWalletStore
import com.gemwallet.android.application.wallet.cases.GetWallets
import com.gemwallet.android.data.service.store.ConfigStore
import com.gemwallet.android.model.BuildInfo
import com.gemwallet.android.model.NotificationsAvailable
import dagger.Lazy
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemDeviceApiClient
import uniffi.gemstone.GemDeviceService
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemSubscriptionService
import javax.inject.Named
import javax.inject.Singleton
import uniffi.gemstone.GemDeviceKeyService


@InstallIn(SingletonComponent::class)
@Module
object DeviceModule {

    @Provides
    @Singleton
    fun provideGemDeviceService(
        @Named("registration") apiClient: GemDeviceApiClient,
        subscriptionService: GemSubscriptionService,
        walletStore: GemstoneWalletStore,
        platform: GemstoneDevicePlatform,
        preferencesService: GemPreferencesService,
    ): GemDeviceService = GemDeviceService(apiClient, subscriptionService, walletStore, platform, preferencesService)

    @Provides
    @Singleton
    fun provideGemSubscriptionService(
        @Named("registration") apiClient: GemDeviceApiClient,
        walletStore: GemstoneWalletStore,
    ): GemSubscriptionService = GemSubscriptionService(apiClient, walletStore)

    @Provides
    @Singleton
    fun provideDevicePlatform(
        @ApplicationContext context: Context,
        buildInfo: BuildInfo,
        deviceService: Lazy<GemDeviceService>,
        deviceKeyService: GemDeviceKeyService,
        preferencesService: GemPreferencesService,
        notificationsAvailable: NotificationsAvailable,
    ): GemstoneDevicePlatform {
        return GemstoneDevicePlatform(
            context = context,
            deviceService = deviceService,
            deviceKeyService = deviceKeyService,
            configStore = ConfigStore(context.getSharedPreferences("device-info", Context.MODE_PRIVATE)),
            requestPushToken = buildInfo.requestPushToken,
            platformStore = buildInfo.platformStore,
            notificationsAvailable = notificationsAvailable,
            versionName = buildInfo.versionName,
            preferencesService = preferencesService,
        )
    }

    @Provides
    fun provideSwitchPushEnabledCase(repository: GemstoneDevicePlatform): SwitchPushEnabled = repository

    @Provides
    fun provideGetPushEnabledCase(repository: GemstoneDevicePlatform): GetPushEnabled = repository

    @Provides
    fun provideSetPushTokenCase(repository: GemstoneDevicePlatform): SetPushToken = repository

    @Provides
    fun provideGetPushTokenCase(repository: GemstoneDevicePlatform): GetPushToken = repository

    @Provides
    fun provideIsDeviceRegisteredCase(repository: GemstoneDevicePlatform): IsDeviceRegistered = repository

    @Provides
    @Singleton
    fun provideDeviceObserverService(
        getWallets: GetWallets,
        deviceService: GemDeviceService,
    ): DeviceObserverService = DeviceObserverService(
        getWallets = getWallets,
        deviceService = deviceService,
    )
}
