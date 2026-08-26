package com.gemwallet.android.data.repositories.di

import android.content.Context
import com.gemwallet.android.application.device.coordinators.GetDeviceId
import com.gemwallet.android.application.session.coordinators.GetCurrentCurrency
import com.gemwallet.android.cases.device.GetPushEnabled
import com.gemwallet.android.cases.device.GetPushToken
import com.gemwallet.android.cases.device.IsDeviceRegistered
import com.gemwallet.android.cases.device.SetPushToken
import com.gemwallet.android.cases.device.SwitchPushEnabled
import com.gemwallet.android.cases.device.SyncDevice
import com.gemwallet.android.data.repositories.device.DeviceObserverService
import com.gemwallet.android.data.repositories.device.DeviceRepository
import com.gemwallet.android.data.repositories.gemstone.GemstoneDeviceStore
import com.gemwallet.android.data.repositories.gemstone.GemstoneWalletStore
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
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


@InstallIn(SingletonComponent::class)
@Module
object DeviceModule {

    @Provides
    @Singleton
    fun provideGemstoneDeviceStore(@ApplicationContext context: Context): GemstoneDeviceStore =
        GemstoneDeviceStore(ConfigStore(context.getSharedPreferences("device-info", Context.MODE_PRIVATE)))

    @Provides
    @Singleton
    fun provideGemDeviceService(
        @Named("registration") apiClient: GemDeviceApiClient,
        subscriptionService: GemSubscriptionService,
        walletsRepository: Lazy<WalletsRepository>,
        deviceStore: GemstoneDeviceStore,
    ): GemDeviceService = GemDeviceService(apiClient, subscriptionService, GemstoneWalletStore(walletsRepository), deviceStore)

    @Provides
    @Singleton
    fun provideGemSubscriptionService(
        @Named("registration") apiClient: GemDeviceApiClient,
        walletsRepository: Lazy<WalletsRepository>,
    ): GemSubscriptionService = GemSubscriptionService(apiClient, GemstoneWalletStore(walletsRepository))

    @Provides
    @Singleton
    fun provideDeviceRepository(
        @ApplicationContext context: Context,
        buildInfo: BuildInfo,
        deviceService: GemDeviceService,
        deviceStore: GemstoneDeviceStore,
        getDeviceId: GetDeviceId,
        preferencesService: GemPreferencesService,
        getCurrentCurrency: GetCurrentCurrency,
        notificationsAvailable: NotificationsAvailable,
    ): DeviceRepository {
        return DeviceRepository(
            context = context,
            deviceService = deviceService,
            deviceStore = deviceStore,
            getDeviceId = getDeviceId,
            configStore = ConfigStore(context.getSharedPreferences("device-info", Context.MODE_PRIVATE)),
            requestPushToken = buildInfo.requestPushToken,
            platformStore = buildInfo.platformStore,
            notificationsAvailable = notificationsAvailable,
            versionName = buildInfo.versionName,
            preferencesService = preferencesService,
            getCurrentCurrency = getCurrentCurrency,
        )
    }

    @Provides
    fun provideSwitchPushEnabledCase(repository: DeviceRepository): SwitchPushEnabled = repository

    @Provides
    fun provideGetPushEnabledCase(repository: DeviceRepository): GetPushEnabled = repository

    @Provides
    fun provideSetPushTokenCase(repository: DeviceRepository): SetPushToken = repository

    @Provides
    fun provideGetPushTokenCase(repository: DeviceRepository): GetPushToken = repository

    @Provides
    fun provideIsDeviceRegisteredCase(repository: DeviceRepository): IsDeviceRegistered = repository

    @Provides
    fun provideSyncDeviceCase(repository: DeviceRepository): SyncDevice = repository

    @Provides
    @Singleton
    fun provideDeviceObserverService(
        walletsRepository: WalletsRepository,
        syncDevice: SyncDevice,
    ): DeviceObserverService = DeviceObserverService(
        walletsRepository = walletsRepository,
        syncDevice = syncDevice,
    )
}
