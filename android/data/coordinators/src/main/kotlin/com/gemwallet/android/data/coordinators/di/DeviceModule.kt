package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.device.cases.SwitchPushEnabled
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemDeviceKeyService
import uniffi.gemstone.GemDeviceKeyServiceInterface
import uniffi.gemstone.GemSecureStore
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object DeviceModule {
    @Provides
    @Singleton
    fun provideDeviceKeyService(secureStore: GemSecureStore): GemDeviceKeyService = GemDeviceKeyService(secureStore)

    @Provides
    @Singleton
    fun provideDeviceKeyServiceInterface(service: GemDeviceKeyService): GemDeviceKeyServiceInterface = service
}
