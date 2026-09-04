package com.gemwallet.android.di

import android.content.Context
import com.gemwallet.android.data.password.TinkGemPreferences
import com.gemwallet.android.data.services.gemstone.stores.GemstonePreferencesStore
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
import uniffi.gemstone.Config
import uniffi.gemstone.AlienProvider
import uniffi.gemstone.GemChartService
import uniffi.gemstone.GemChartServiceInterface
import uniffi.gemstone.GemExplorerService
import uniffi.gemstone.GemPriceAlertService
import uniffi.gemstone.GemConfigService
import uniffi.gemstone.GemAuthService
import uniffi.gemstone.GemBalanceService
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.data.services.gemstone.stores.GemstoneKeystorePassword
import uniffi.gemstone.GemKeystore
import uniffi.gemstone.GemDeviceApiClient as GemstoneDeviceApiClient
import uniffi.gemstone.GemFiatQuoteService
import uniffi.gemstone.GemFiatQuoteServiceInterface
import uniffi.gemstone.GemFiatService
import uniffi.gemstone.GemFiatServiceInterface
import uniffi.gemstone.GemFiatStore
import com.gemwallet.android.data.services.gemstone.stores.GemstonePerpetualStore
import uniffi.gemstone.GemAssetStore
import uniffi.gemstone.GemAssetsService
import uniffi.gemstone.GemPriceService
import uniffi.gemstone.GemSearchStore
import uniffi.gemstone.GemSearchService
import uniffi.gemstone.GemBannerService
import uniffi.gemstone.GemAppStartService
import uniffi.gemstone.GemAppStartServiceInterface
import uniffi.gemstone.GemPerpetualService
import uniffi.gemstone.GemPortfolioService
import uniffi.gemstone.GemPortfolioServiceInterface
import uniffi.gemstone.GemPortfolioStore
import uniffi.gemstone.GemRewardsService
import uniffi.gemstone.GemRewardsServiceInterface
import uniffi.gemstone.GemWalletSessionService
import uniffi.gemstone.GemSupportService
import uniffi.gemstone.GemSupportServiceInterface
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
import uniffi.gemstone.GemPaymentService
import uniffi.gemstone.GemServiceStatus
import uniffi.gemstone.serviceStatusTimeoutSeconds
import uniffi.gemstone.GemSimulationService
import uniffi.gemstone.GemSimulationServiceInterface
import javax.inject.Singleton
import uniffi.gemstone.GemFileStore
import uniffi.gemstone.GemWalletPreferencesService
import uniffi.gemstone.GemWalletService
import uniffi.gemstone.GemDeviceService
import uniffi.gemstone.GemNodeServiceInterface
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
        nodeService: GemNodeServiceInterface,
        okHttpClient: OkHttpClient,
    ): AlienProvider {
        return NativeProvider(
            nodeService = nodeService,
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
        GemstoneApiClient(alienProvider)

    @Provides
    @Singleton
    @Named("registration")
    fun provideDeviceRegistrationApiClient(
        alienProvider: AlienProvider,
        deviceKeyService: GemDeviceKeyService,
    ): GemstoneDeviceApiClient = GemstoneDeviceApiClient(alienProvider, deviceKeyService)

    @Provides
    @Singleton
    fun provideGemstoneDeviceApiClient(
        alienProvider: AlienProvider,
        deviceKeyService: GemDeviceKeyService,
        deviceService: Lazy<GemDeviceService>,
    ): GemstoneDeviceApiClient = GemstoneDeviceApiClient(alienProvider, deviceKeyService)
        .apply { setDeviceSyncPreflight(deviceService.get()) }



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
        deviceKeyService,
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
        balanceService: GemBalanceService,
        walletConfigurationService: GemWalletConfigurationService,
        walletService: GemWalletService,
        deviceService: GemDeviceService,
    ): GemAppStartService = GemAppStartService(configService, bannerService, assetsService, balanceService, walletConfigurationService, walletService, deviceService)

    @Provides
    fun provideGemAppStartServiceInterface(service: GemAppStartService): GemAppStartServiceInterface = service


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
        sessionService: GemWalletSessionService,
    ): GemRewardsServiceInterface = GemRewardsService(apiClient, authService, balanceService, sessionService)


    @Provides
    @Singleton
    fun provideGemFiatService(
        apiClient: GemstoneDeviceApiClient,
        assetsService: GemAssetsService,
        store: GemFiatStore,
    ): GemFiatService = GemFiatService(apiClient, assetsService, store)

    @Provides
    @Singleton
    fun provideGemFiatServiceInterface(service: GemFiatService): GemFiatServiceInterface = service

    @Provides
    fun provideGemFiatQuoteService(
        fiatService: GemFiatService,
        balanceService: GemBalanceService,
        walletSessionService: GemWalletSessionService,
    ): GemFiatQuoteServiceInterface = GemFiatQuoteService(fiatService, balanceService, walletSessionService)


    @Provides
    @Singleton
    fun provideGemPortfolioService(
        apiClient: GemstoneDeviceApiClient,
        store: GemPortfolioStore,
        priceService: GemPriceService,
        perpetualService: GemPerpetualService,
        preferencesService: GemPreferencesService,
    ): GemPortfolioService = GemPortfolioService(apiClient, store, priceService, perpetualService, preferencesService)

    @Provides
    fun provideGemPortfolioServiceInterface(service: GemPortfolioService): GemPortfolioServiceInterface = service

    @Provides
    @Singleton
    fun provideGemStaticApiClient(alienProvider: AlienProvider): GemStaticApiClient =
        GemStaticApiClient(alienProvider)


    @Provides
    @Singleton
    fun provideGemChartService(
        apiClient: GemstoneApiClient,
        priceService: GemPriceService,
        preferencesService: GemPreferencesService,
        priceAlertService: GemPriceAlertService,
        explorerService: GemExplorerService,
    ): GemChartService = GemChartService(apiClient, priceService, preferencesService, priceAlertService, explorerService)

    @Provides
    @Singleton
    fun provideGemConfigService(apiClient: GemstoneApiClient, preferencesService: GemPreferencesService): GemConfigService = GemConfigService(apiClient, preferencesService)



    @Provides
    @Singleton
    fun provideGemScanService(
        okHttpClient: OkHttpClient,
        nodeService: GemNodeServiceInterface,
        deviceKeyService: GemDeviceKeyService,
    ): GemScanService = GemScanService(
        GemstoneDeviceApiClient(
            NativeProvider(
                nodeService = nodeService,
                httpClient = okHttpClient.newBuilder()
                    .callTimeout(Config().getScanConfig().timeoutSeconds.toLong(), TimeUnit.SECONDS)
                    .build(),
            ),
            deviceKeyService,
        )
    )



    @Provides
    @Singleton
    fun provideGemPaymentService(alienProvider: AlienProvider): GemPaymentService = GemPaymentService(alienProvider)


    @Provides
    @Singleton
    fun provideGemServiceStatus(
        nodeService: GemNodeServiceInterface,
        okHttpClient: OkHttpClient,
    ): GemServiceStatus {
        val httpClient = okHttpClient.newBuilder()
            .callTimeout(serviceStatusTimeoutSeconds().toLong(), TimeUnit.SECONDS)
            .build()
        val provider = NativeProvider(
            nodeService = nodeService,
            httpClient = httpClient,
        )
        return GemServiceStatus(provider)
    }

    @Provides
    @Singleton
    fun provideGemGemSimulationService(
        alienProvider: AlienProvider,
    ): GemSimulationService = GemSimulationService(alienProvider)

    @Provides
    @Singleton
    fun provideGemSimulationServiceInterface(
        service: GemSimulationService,
    ): GemSimulationServiceInterface = service

    @Provides
    fun provideGemSupportServiceInterface(service: GemSupportService): GemSupportServiceInterface = service

    @Provides
    fun provideGemChartServiceInterface(service: GemChartService): GemChartServiceInterface = service
}
