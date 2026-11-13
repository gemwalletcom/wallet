package com.gemwallet.android.data.services.gemstone.di

import android.content.Context
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.lightning.cases.OpenLightning
import com.gemwallet.android.data.services.gemstone.lightning.LightningSessionObserver
import com.gemwallet.android.data.services.gemstone.stores.DeviceAuthentication
import com.gemwallet.android.data.services.gemstone.stores.GemstoneKeystorePassword
import com.gemwallet.android.data.services.gemstone.stores.GemstoneWalletStore
import com.gemwallet.android.data.services.gemstone.stores.LightningKeystorePassword
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemAuthService
import uniffi.gemstone.GemAvatarService
import uniffi.gemstone.GemBalanceService
import uniffi.gemstone.GemConfirmService
import uniffi.gemstone.GemConfirmServiceInterface
import uniffi.gemstone.GemConfirmTransferService
import uniffi.gemstone.GemDeviceApiClient
import uniffi.gemstone.GemDeviceKeyService
import uniffi.gemstone.GemExplorerService
import uniffi.gemstone.GemFileStore
import uniffi.gemstone.GemKeystore
import uniffi.gemstone.GemLightningService
import uniffi.gemstone.GemLightningServiceInterface
import uniffi.gemstone.GemNameService
import uniffi.gemstone.GemPaymentService
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemRecentActivityService
import uniffi.gemstone.GemSignMessageService
import uniffi.gemstone.GemSwapService
import uniffi.gemstone.GemSwapServiceInterface
import uniffi.gemstone.GemTransactionsService
import uniffi.gemstone.GemWalletPreferencesService
import uniffi.gemstone.GemWalletService
import uniffi.gemstone.GemWalletSessionService
import uniffi.gemstone.GemWalletSessionServiceInterface
import java.io.File
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object KeystoreModule {

    @Provides
    @Singleton
    fun provideGemKeystore(@ApplicationContext context: Context): GemKeystore = GemKeystore(context.dataDir.toString())

    @Provides
    @Singleton
    fun provideGemWalletService(
        keystore: GemKeystore,
        passwordStore: PasswordStore,
        deviceAuthentication: DeviceAuthentication,
        walletStore: GemstoneWalletStore,
        walletSessionService: GemWalletSessionService,
        preferencesService: GemPreferencesService,
        fileStore: GemFileStore,
        walletPreferencesService: GemWalletPreferencesService,
        explorerService: GemExplorerService,
        nameService: GemNameService,
        avatarService: GemAvatarService,
        lightningService: GemLightningService,
    ): GemWalletService = GemWalletService(
        keystore,
        GemstoneKeystorePassword(passwordStore, deviceAuthentication),
        walletStore,
        walletSessionService,
        preferencesService,
        fileStore,
        walletPreferencesService,
        explorerService,
        nameService,
        avatarService,
        lightningService,
    )

    @Provides
    @Singleton
    fun provideGemConfirmTransferService(
        confirmService: GemConfirmServiceInterface,
        explorerService: GemExplorerService,
        nameService: GemNameService,
        keystore: GemKeystore,
        passwordStore: PasswordStore,
        deviceAuthentication: DeviceAuthentication,
        recentActivity: GemRecentActivityService,
        preferencesService: GemPreferencesService,
        paymentService: GemPaymentService,
        swapService: GemSwapServiceInterface,
    ): GemConfirmTransferService = GemConfirmTransferService(
        confirmService as GemConfirmService,
        explorerService,
        nameService,
        keystore,
        GemstoneKeystorePassword(passwordStore, deviceAuthentication),
        recentActivity,
        preferencesService,
        paymentService,
        swapService as GemSwapService,
    )

    @Provides
    fun provideGemSignMessageService(names: GemNameService, explorer: GemExplorerService, keystore: GemKeystore, passwordStore: PasswordStore, deviceAuthentication: DeviceAuthentication): GemSignMessageService =
        GemSignMessageService(names, explorer, keystore, GemstoneKeystorePassword(passwordStore, deviceAuthentication))

    @Provides
    @Singleton
    fun provideGemAuthService(apiClient: GemDeviceApiClient, keystore: GemKeystore, passwordStore: PasswordStore, deviceAuthentication: DeviceAuthentication, deviceKeyService: GemDeviceKeyService): GemAuthService = GemAuthService(
        apiClient,
        keystore,
        GemstoneKeystorePassword(passwordStore, deviceAuthentication),
        deviceKeyService,
    )

    @Provides
    @Singleton
    fun provideGemLightningService(
        @ApplicationContext context: Context,
        keystore: GemKeystore,
        passwordStore: PasswordStore,
        deviceAuthentication: DeviceAuthentication,
        balanceService: GemBalanceService,
        transactionsService: GemTransactionsService,
        walletSessionService: GemWalletSessionService,
    ): GemLightningService = GemLightningService(
        keystore,
        LightningKeystorePassword(GemstoneKeystorePassword(passwordStore, deviceAuthentication)),
        File(context.noBackupFilesDir, "lightning").absolutePath,
        balanceService,
        transactionsService,
        walletSessionService,
    )

    @Provides
    fun provideGemLightningServiceInterface(service: GemLightningService): GemLightningServiceInterface = service

    @Provides
    @Singleton
    fun provideLightningSessionObserver(service: GemLightningServiceInterface, session: GemWalletSessionServiceInterface): LightningSessionObserver = LightningSessionObserver(
        service = service,
        session = session,
    )

    @Provides
    fun provideOpenLightning(observer: LightningSessionObserver): OpenLightning = observer
}
