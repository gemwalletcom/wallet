package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.device.cases.EnableDevicePush
import com.gemwallet.android.application.device.cases.SwitchPushEnabled
import com.gemwallet.android.data.coordinators.device.EnableDevicePushImpl
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton
import uniffi.gemstone.GemDeviceKeyService
import uniffi.gemstone.GemSecureStore

@InstallIn(SingletonComponent::class)
@Module
object DeviceModule {
    @Provides
    @Singleton
    fun provideDeviceKeyService(secureStore: GemSecureStore): GemDeviceKeyService = GemDeviceKeyService(secureStore)

    @Provides
    @Singleton
    fun provideEnableDevicePush(
        switchPushEnabled: SwitchPushEnabled,
    ): EnableDevicePush = EnableDevicePushImpl(switchPushEnabled)
}