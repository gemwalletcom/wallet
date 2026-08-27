package com.gemwallet.android.di

import android.content.Context
import com.gemwallet.android.Constants
import com.gemwallet.android.cases.nodes.GetNodeUrlCase
import com.gemwallet.android.data.password.TinkGemPreferences
import com.gemwallet.android.data.repositories.gemstone.GemstonePreferencesStore
import com.gemwallet.android.cases.device.SyncDevice
import com.gemwallet.android.services.DeviceSyncPreflight
import com.gemwallet.android.application.device.coordinators.GetDeviceId
import com.gemwallet.android.math.fromHex
import kotlinx.coroutines.runBlocking
import com.gemwallet.android.data.services.gemapi.NativeProvider
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
import com.gemwallet.android.data.repositories.gemstone.GemstoneKeystorePassword
import uniffi.gemstone.GemKeystore
import uniffi.gemstone.GemDeviceApiClient as GemstoneDeviceApiClient
import uniffi.gemstone.GemFiatService
import uniffi.gemstone.GemFiatStore
import com.gemwallet.android.data.repositories.gemstone.GemstonePerpetualStore
import uniffi.gemstone.GemAssetStore
import uniffi.gemstone.GemAssetsService
import uniffi.gemstone.GemPriceService
import uniffi.gemstone.GemSearchStore
import uniffi.gemstone.GemSearchService
import uniffi.gemstone.GemBannerService
import uniffi.gemstone.GemAppStartService
import uniffi.gemstone.GemPortfolioService
import uniffi.gemstone.GemRewardsService
import uniffi.gemstone.GemSupportService
import uniffi.gemstone.GemSupportStore
import uniffi.gemstone.GemWalletConfigurationService
import uniffi.gemstone.GemBannerStore
import com.gemwallet.android.data.repositories.gemstone.GemstonePortfolioStore
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import javax.inject.Named
import uniffi.gemstone.GemGateway
import uniffi.gemstone.GemApiClient as GemstoneApiClient
import uniffi.gemstone.GemScanService
import uniffi.gemstone.GemStaticApiClient
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemPreferencesStore
import uniffi.gemstone.PaymentService
import uniffi.gemstone.PaymentServiceInterface
import uniffi.gemstone.GemServiceStatus
import uniffi.gemstone.serviceStatusTimeoutSeconds
import uniffi.gemstone.TransactionSimulationService
import uniffi.gemstone.TransactionSimulationServiceInterface
import javax.inject.Singleton
import uniffi.gemstone.GemFileStore
import uniffi.gemstone.GemWalletPreferencesService
import uniffi.gemstone.GemWalletService

@InstallIn(SingletonComponent::class)
@Module
object GatewayModule {

    @Singleton
    @Provides
    fun provideGemPreferencesStore(@ApplicationContext context: Context): GemPreferencesStore = TinkGemPreferences(context)


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
        securePreferences: GemPreferencesStore,
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
        getDeviceId: GetDeviceId,
    ): GemstoneDeviceApiClient = GemstoneDeviceApiClient(
        alienProvider,
        Constants.API_URL,
        runBlocking { getDeviceId.getDeviceKey().fromHex() },
    )

    @Provides
    @Singleton
    fun provideGemstoneDeviceApiClient(
        alienProvider: AlienProvider,
        getDeviceId: GetDeviceId,
        syncDevice: Lazy<SyncDevice>,
    ): GemstoneDeviceApiClient = GemstoneDeviceApiClient.withPreflight(
        alienProvider,
        Constants.API_URL,
        runBlocking { getDeviceId.getDeviceKey().fromHex() },
        DeviceSyncPreflight(syncDevice),
    )



    @Provides
    @Singleton
    fun provideGemAuthService(
        apiClient: GemstoneDeviceApiClient,
        keystore: GemKeystore,
        passwordStore: PasswordStore,
        getDeviceId: GetDeviceId,
    ): GemAuthService = GemAuthService(
        apiClient,
        keystore,
        GemstoneKeystorePassword(passwordStore),
        runBlocking { getDeviceId.getDeviceKey().fromHex() },
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
    fun provideGemPortfolioService(apiClient: GemstoneDeviceApiClient, assetsRepository: Lazy<AssetsRepository>): GemPortfolioService =
        GemPortfolioService(apiClient, GemstonePortfolioStore(assetsRepository))

    @Provides
    @Singleton
    fun provideGemStaticApiClient(alienProvider: AlienProvider): GemStaticApiClient =
        GemStaticApiClient(alienProvider, Constants.ASSETS_URL)


    @Provides
    @Singleton
    fun provideGemChartService(apiClient: GemstoneApiClient): GemChartService = GemChartService(apiClient)

    @Provides
    @Singleton
    fun provideGemConfigService(apiClient: GemstoneApiClient, preferencesService: GemPreferencesService): GemConfigService = GemConfigService(apiClient, preferencesService)



    @Provides
    @Singleton
    fun provideGemScanService(apiClient: GemstoneDeviceApiClient): GemScanService = GemScanService(apiClient)



    @Provides
    @Singleton
    fun providePaymentService(alienProvider: AlienProvider): PaymentServiceInterface = PaymentService(alienProvider)


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
