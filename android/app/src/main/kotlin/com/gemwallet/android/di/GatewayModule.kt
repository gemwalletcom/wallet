package com.gemwallet.android.di

import android.content.Context
import com.gemwallet.android.Constants
import com.gemwallet.android.cases.nodes.GetNodeUrlCase
import com.gemwallet.android.data.password.TinkGemPreferences
import com.gemwallet.android.data.services.gemstone.stores.GemstonePreferencesStore
import com.gemwallet.android.services.DeviceSyncPreflight
import com.gemwallet.android.math.fromHex
import kotlinx.coroutines.runBlocking
import com.gemwallet.android.data.services.nativeprovider.NativeProvider
import dagger.Lazy
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import okhttp3.OkHttpClient
import java.util.concurrent.TimeUnit
import uniffi.gemstone.AlienProvider
import uniffi.gemstone.GemChartService
import uniffi.gemstone.GemConfigService
import uniffi.gemstone.GemAuthService
import uniffi.gemstone.GemBalanceService
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.data.services.gemstone.stores.GemstoneKeystorePassword
import uniffi.gemstone.GemKeystore
import uniffi.gemstone.GemDeviceApiClient as GemstoneDeviceApiClient
import uniffi.gemstone.GemFiatService
import uniffi.gemstone.GemFiatStore
import com.gemwallet.android.data.services.gemstone.stores.GemstonePerpetualStore
import uniffi.gemstone.GemAssetStore
import uniffi.gemstone.GemAssetsService
import uniffi.gemstone.GemPriceService
import uniffi.gemstone.GemSearchStore
import uniffi.gemstone.GemSearchService
import uniffi.gemstone.GemBannerService
import uniffi.gemstone.GemAppStartService
import uniffi.gemstone.GemPerpetualService
import uniffi.gemstone.GemPortfolioService
import uniffi.gemstone.GemPortfolioStore
import uniffi.gemstone.GemRewardsService
import uniffi.gemstone.GemSupportService
import uniffi.gemstone.GemSupportStore
import uniffi.gemstone.GemWalletConfigurationService
import uniffi.gemstone.GemBannerStore
import javax.inject.Named
import uniffi.gemstone.GemGateway
import uniffi.gemstone.GemApiClient as GemstoneApiClient
import uniffi.gemstone.GemScanService
import uniffi.gemstone.GemStaticApiClient
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemPreferencesStore
import uniffi.gemstone.GemSecureStore
import uniffi.gemstone.GemPaymentLinkService
import uniffi.gemstone.GemPaymentLinkServiceInterface
import uniffi.gemstone.GemPaymentService
import uniffi.gemstone.GemServiceStatus
import uniffi.gemstone.serviceStatusTimeoutSeconds
import uniffi.gemstone.TransactionSimulationService
import uniffi.gemstone.TransactionSimulationServiceInterface
import javax.inject.Singleton
import uniffi.gemstone.GemFileStore
import uniffi.gemstone.GemWalletPreferencesService
import uniffi.gemstone.GemWalletService
import uniffi.gemstone.GemDeviceService
import uniffi.gemstone.GemDeviceKeyService

@InstallIn(SingletonComponent::class)
@Module
object GatewayModule {

    @Singleton
    @Provides
    fun provideGemSecureStore(@ApplicationContext context: Context): GemSecureStore = TinkGemPreferences(context)


    @Singleton
    @Provides
    fun provideAlienProvider(
        getNodeUrlCase: GetNodeUrlCase,
        okHttpClient: OkHttpClient,
    ): AlienProvider {
        return NativeProvider(
            getNodeUrlCase = getNodeUrlCase,
            httpClient = okHttpClient,
        )
    }

    @Provides
    @Singleton
    fun provideGateway(
        alienProvider: AlienProvider,
        securePreferences: GemSecureStore,
        @ApplicationContext context: Context,
    ): GemGateway {
        return GemGateway(
            alienProvider,
            preferences = GemstonePreferencesStore(
                sharedPreferences = context.getSharedPreferences("gateway_preferences", Context.MODE_PRIVATE)
            ),
            securePreferences = securePreferences,
        )
    }


    @Provides
    @Singleton
    fun provideGemstoneApiClient(alienProvider: AlienProvider): GemstoneApiClient =
        GemstoneApiClient(alienProvider, Constants.API_URL)

    @Provides
    @Singleton
    @Named("registration")
    fun provideDeviceRegistrationApiClient(
        alienProvider: AlienProvider,
        deviceKeyService: GemDeviceKeyService,
    ): GemstoneDeviceApiClient = GemstoneDeviceApiClient(
        alienProvider,
        Constants.API_URL,
        deviceKeyService.keyPair().privateKey,
    )

    @Provides
    @Singleton
    fun provideGemstoneDeviceApiClient(
        alienProvider: AlienProvider,
        deviceKeyService: GemDeviceKeyService,
        deviceService: Lazy<GemDeviceService>,
    ): GemstoneDeviceApiClient = GemstoneDeviceApiClient.withPreflight(
        alienProvider,
        Constants.API_URL,
        deviceKeyService.keyPair().privateKey,
        DeviceSyncPreflight(deviceService),
    )



    @Provides
    @Singleton
    fun provideGemAuthService(
        apiClient: GemstoneDeviceApiClient,
        keystore: GemKeystore,
        passwordStore: PasswordStore,
        deviceKeyService: GemDeviceKeyService,
    ): GemAuthService = GemAuthService(
        apiClient,
        keystore,
        GemstoneKeystorePassword(passwordStore),
        deviceKeyService.keyPair().privateKey,
    )


    @Provides
    @Singleton
    fun provideGemWalletConfigurationService(
        apiClient: GemstoneDeviceApiClient,
        bannerStore: GemBannerStore,
        walletPreferencesService: GemWalletPreferencesService,
    ): GemWalletConfigurationService = GemWalletConfigurationService(apiClient, bannerStore, walletPreferencesService)

    @Provides
    @Singleton
    fun provideGemSearchService(
        assetsService: GemAssetsService,
        assetStore: GemAssetStore,
        priceService: GemPriceService,
        perpetualStore: GemstonePerpetualStore,
        searchStore: GemSearchStore,
    ): GemSearchService = GemSearchService(assetsService, assetStore, priceService, perpetualStore, searchStore)

    @Provides
    @Singleton
    fun provideGemAppStartService(
        configService: GemConfigService,
        bannerService: GemBannerService,
        assetsService: GemAssetsService,
        walletConfigurationService: GemWalletConfigurationService,
        walletService: GemWalletService,
    ): GemAppStartService = GemAppStartService(configService, bannerService, assetsService, walletConfigurationService, walletService)


    @Provides
    @Singleton
    fun provideGemSupportService(
        apiClient: GemstoneDeviceApiClient,
        store: GemSupportStore,
        fileStore: GemFileStore,
        alienProvider: AlienProvider,
    ): GemSupportService = GemSupportService(apiClient, store, fileStore, alienProvider)

    @Provides
    @Singleton
    fun provideGemRewardsService(
        apiClient: GemstoneDeviceApiClient,
        authService: GemAuthService,
        balanceService: GemBalanceService,
    ): GemRewardsService = GemRewardsService(apiClient, authService, balanceService)


    @Provides
    @Singleton
    fun provideGemFiatService(
        apiClient: GemstoneDeviceApiClient,
        assetsService: GemAssetsService,
        store: GemFiatStore,
    ): GemFiatService = GemFiatService(apiClient, assetsService, store)


    @Provides
    @Singleton
    fun provideGemPortfolioService(
        apiClient: GemstoneDeviceApiClient,
        store: GemPortfolioStore,
        priceService: GemPriceService,
        perpetualService: GemPerpetualService,
    ): GemPortfolioService = GemPortfolioService(apiClient, store, priceService, perpetualService)

    @Provides
    @Singleton
    fun provideGemStaticApiClient(alienProvider: AlienProvider): GemStaticApiClient =
        GemStaticApiClient(alienProvider, Constants.ASSETS_URL)


    @Provides
    @Singleton
    fun provideGemChartService(apiClient: GemstoneApiClient, priceService: GemPriceService): GemChartService = GemChartService(apiClient, priceService)

    @Provides
    @Singleton
    fun provideGemConfigService(apiClient: GemstoneApiClient, preferencesService: GemPreferencesService): GemConfigService = GemConfigService(apiClient, preferencesService)



    @Provides
    @Singleton
    fun provideGemScanService(apiClient: GemstoneDeviceApiClient): GemScanService = GemScanService(apiClient)



    @Provides
    @Singleton
    fun providePaymentLinkService(alienProvider: AlienProvider): GemPaymentLinkServiceInterface = GemPaymentLinkService(alienProvider)

    @Provides
    @Singleton
    fun provideGemPaymentService(): GemPaymentService = GemPaymentService()


    @Provides
    @Singleton
    fun provideGemServiceStatus(
        getNodeUrlCase: GetNodeUrlCase,
        okHttpClient: OkHttpClient,
    ): GemServiceStatus {
        val httpClient = okHttpClient.newBuilder()
            .callTimeout(serviceStatusTimeoutSeconds().toLong(), TimeUnit.SECONDS)
            .build()
        val provider = NativeProvider(
            getNodeUrlCase = getNodeUrlCase,
            httpClient = httpClient,
        )
        return GemServiceStatus(provider)
    }

    @Provides
    @Singleton
    fun provideGemTransactionSimulationService(
        alienProvider: AlienProvider,
    ): TransactionSimulationService = TransactionSimulationService(alienProvider)

    @Provides
    @Singleton
    fun provideTransactionSimulationServiceInterface(
        service: TransactionSimulationService,
    ): TransactionSimulationServiceInterface = service
}
