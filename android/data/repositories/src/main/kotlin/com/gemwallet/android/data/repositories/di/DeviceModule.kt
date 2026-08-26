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
import com.gemwallet.android.data.repositories.pricealerts.PriceAlertRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.data.service.store.ConfigStore
import com.gemwallet.android.model.BuildInfo
import com.gemwallet.android.model.NotificationsAvailable
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemDeviceService
import uniffi.gemstone.GemSubscriptionService
import javax.inject.Singleton


@InstallIn(SingletonComponent::class)
@Module
object DeviceModule {

    @Provides
    @Singleton
    fun provideDeviceRepository(
        @ApplicationContext context: Context,
        buildInfo: BuildInfo,
        deviceService: GemDeviceService,
        subscriptionService: GemSubscriptionService,
        getDeviceId: GetDeviceId,
        priceAlertRepository: PriceAlertRepository,
        getCurrentCurrency: GetCurrentCurrency,
        walletsRepository: WalletsRepository,
        notificationsAvailable: NotificationsAvailable,
    ): DeviceRepository {
        return DeviceRepository(
            context = context,
            deviceService = deviceService,
            subscriptionService = subscriptionService,
            getDeviceId = getDeviceId,
            configStore = ConfigStore(context.getSharedPreferences("device-info", Context.MODE_PRIVATE)),
            requestPushToken = buildInfo.requestPushToken,
            platformStore = buildInfo.platformStore,
            notificationsAvailable = notificationsAvailable,
            versionName = buildInfo.versionName,
            priceAlertRepository = priceAlertRepository,
            getCurrentCurrency = getCurrentCurrency,
            walletsRepository = walletsRepository,
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
