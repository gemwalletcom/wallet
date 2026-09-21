package com.gemwallet.android.data.services.gemstone.di

import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemAddressService
import uniffi.gemstone.GemApplicationMetadataService
import uniffi.gemstone.GemApplicationMetadataServiceInterface
import uniffi.gemstone.GemAssetConfigService
import uniffi.gemstone.GemAssetConfigServiceInterface
import uniffi.gemstone.GemConnectionService
import uniffi.gemstone.GemConnectionServiceInterface
import uniffi.gemstone.GemSecurityService
import uniffi.gemstone.GemSecurityServiceInterface
import uniffi.gemstone.GemSimulationFormatter
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
    fun providePriceAlertFormatter(): PriceAlertFormatter = PriceAlertFormatter()

    @Provides
    fun provideGemAssetConfigServiceInterface(service: GemAssetConfigService): GemAssetConfigServiceInterface = service

    @Provides
    @Singleton
    fun provideConnectionServiceInterface(service: GemConnectionService): GemConnectionServiceInterface = service

    @Provides
    @Singleton
    fun provideSecurityServiceInterface(service: GemSecurityService): GemSecurityServiceInterface = service

    @Provides
    @Singleton
    fun provideGemApplicationMetadataService(): GemApplicationMetadataService = GemApplicationMetadataService()

    @Provides
    @Singleton
    fun provideApplicationMetadataServiceInterface(service: GemApplicationMetadataService): GemApplicationMetadataServiceInterface = service
}
