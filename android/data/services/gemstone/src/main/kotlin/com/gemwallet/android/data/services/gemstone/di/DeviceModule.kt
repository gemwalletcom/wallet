package com.gemwallet.android.data.services.gemstone.di

import android.content.Context
import com.gemwallet.android.application.device.cases.GetPushEnabled
import com.gemwallet.android.application.device.cases.GetPushToken
import com.gemwallet.android.application.device.cases.SetPushToken
import com.gemwallet.android.application.device.cases.SwitchPushEnabled
import com.gemwallet.android.data.services.gemstone.device.DeviceObserverService
import com.gemwallet.android.data.services.gemstone.device.DevicePushSettings
import com.gemwallet.android.data.services.gemstone.device.GemstoneDevicePlatform
import com.gemwallet.android.data.services.gemstone.stores.GemstoneWalletStore
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
    fun provideDevicePushSettings(
        @ApplicationContext context: Context,
        notificationsAvailable: NotificationsAvailable,
        preferencesService: GemPreferencesService,
        deviceService: Lazy<GemDeviceService>,
    ): DevicePushSettings = DevicePushSettings(
        context = context,
        configStore = ConfigStore(context.getSharedPreferences("device-info", Context.MODE_PRIVATE)),
        notificationsAvailable = notificationsAvailable,
        preferencesService = preferencesService,
        deviceService = deviceService,
    )

    @Provides
    @Singleton
    fun provideDevicePlatform(
        @ApplicationContext context: Context,
        buildInfo: BuildInfo,
        deviceKeyService: GemDeviceKeyService,
        preferencesService: GemPreferencesService,
        notificationsAvailable: NotificationsAvailable,
        pushSettings: DevicePushSettings,
    ): GemstoneDevicePlatform {
        return GemstoneDevicePlatform(
            context = context,
            deviceKeyService = deviceKeyService,
            getPushToken = pushSettings,
            setPushToken = pushSettings,
            requestPushToken = buildInfo.requestPushToken,
            platformStore = buildInfo.platformStore,
            notificationsAvailable = notificationsAvailable,
            versionName = buildInfo.versionName,
            preferencesService = preferencesService,
        )
    }

    @Provides
    fun provideSwitchPushEnabledCase(pushSettings: DevicePushSettings): SwitchPushEnabled = pushSettings

    @Provides
    fun provideGetPushEnabledCase(pushSettings: DevicePushSettings): GetPushEnabled = pushSettings

    @Provides
    fun provideSetPushTokenCase(pushSettings: DevicePushSettings): SetPushToken = pushSettings

    @Provides
    fun provideGetPushTokenCase(pushSettings: DevicePushSettings): GetPushToken = pushSettings

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
