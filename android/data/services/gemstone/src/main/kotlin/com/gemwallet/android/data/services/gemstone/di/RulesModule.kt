package com.gemwallet.android.data.services.gemstone.di

import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemAddressService
import uniffi.gemstone.GemAmountService
import uniffi.gemstone.GemApplicationMetadataService
import uniffi.gemstone.GemAssetConfigService
import uniffi.gemstone.GemFeeService
import uniffi.gemstone.GemNameService
import uniffi.gemstone.GemRecipientService
import uniffi.gemstone.GemSimulationFormatter
import uniffi.gemstone.GemSwapQuoteService
import uniffi.gemstone.GemTransactionFormatter
import uniffi.gemstone.GemTransferService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object RulesModule {

    @Provides
    @Singleton
    fun provideGemAddressService(): GemAddressService = GemAddressService()

    @Provides
    @Singleton
    fun provideGemAmountService(): GemAmountService = GemAmountService()

    @Provides
    @Singleton
    fun provideGemApplicationMetadataService(): GemApplicationMetadataService = GemApplicationMetadataService()

    @Provides
    @Singleton
    fun provideGemAssetConfigService(): GemAssetConfigService = GemAssetConfigService()

    @Provides
    @Singleton
    fun provideGemFeeService(): GemFeeService = GemFeeService()

    @Provides
    @Singleton
    fun provideGemRecipientService(nameService: GemNameService): GemRecipientService = nameService.recipients()

    @Provides
    @Singleton
    fun provideGemSimulationFormatter(): GemSimulationFormatter = GemSimulationFormatter()

    @Provides
    @Singleton
    fun provideGemSwapQuoteService(): GemSwapQuoteService = GemSwapQuoteService()

    @Provides
    @Singleton
    fun provideGemTransactionFormatter(): GemTransactionFormatter = GemTransactionFormatter()

    @Provides
    @Singleton
    fun provideGemTransferService(): GemTransferService = GemTransferService()
}
