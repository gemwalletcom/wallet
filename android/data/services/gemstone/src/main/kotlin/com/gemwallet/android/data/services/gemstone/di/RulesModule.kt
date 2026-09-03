package com.gemwallet.android.data.services.gemstone.di

import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemAddressService
import uniffi.gemstone.BalanceCalculator
import uniffi.gemstone.GemAssetConfigService
import uniffi.gemstone.GemAssetConfigServiceInterface
import uniffi.gemstone.GemConnectionService
import uniffi.gemstone.GemSecurityService
import uniffi.gemstone.GemSimulationFormatter
import uniffi.gemstone.GemTransferService
import uniffi.gemstone.GemTransferServiceInterface
import uniffi.gemstone.PriceAlertFormatter
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object RulesModule {

    @Provides
    @Singleton
    fun provideGemAddressService(): GemAddressService = GemAddressService()

    @Provides
    @Singleton
    fun provideBalanceCalculator(): BalanceCalculator = BalanceCalculator()

    @Provides
    @Singleton
    fun provideGemAssetConfigService(): GemAssetConfigService = GemAssetConfigService()

    @Provides
    @Singleton
    fun provideGemConnectionService(): GemConnectionService = GemConnectionService()


    @Provides
    @Singleton
    fun provideGemSecurityService(): GemSecurityService = GemSecurityService()

    @Provides
    @Singleton
    fun provideGemSimulationFormatter(): GemSimulationFormatter = GemSimulationFormatter()

    @Provides
    @Singleton
    fun provideGemTransferService(): GemTransferService = GemTransferService()

    @Provides
    @Singleton
    fun providePriceAlertFormatter(): PriceAlertFormatter = PriceAlertFormatter()

    @Provides
    fun provideGemTransferServiceInterface(service: GemTransferService): GemTransferServiceInterface = service

    @Provides
    fun provideGemAssetConfigServiceInterface(service: GemAssetConfigService): GemAssetConfigServiceInterface = service
}
