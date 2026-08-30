package com.gemwallet.android.di

import android.content.Context
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.SecurityStore
import com.gemwallet.android.application.wallet_import.cases.SyncWalletImport
import com.gemwallet.android.blockchain.operators.CreateWalletOperator
import com.gemwallet.android.blockchain.operators.LoadPrivateDataOperator
import com.gemwallet.android.blockchain.operators.MigrateKeystoreOperator
import com.gemwallet.android.blockchain.operators.ValidatePhraseOperator
import com.gemwallet.android.blockchain.operators.gemstone.GemCreateWalletOperator
import com.gemwallet.android.blockchain.operators.gemstone.GemLoadPrivateDataOperator
import com.gemwallet.android.blockchain.operators.gemstone.GemMigrateKeystoreOperator
import com.gemwallet.android.blockchain.operators.gemstone.GemValidatePhraseOperator
import com.gemwallet.android.blockchain.services.GemSignMessageOperator
import com.gemwallet.android.blockchain.services.KeystoreTransactionSigner
import uniffi.gemstone.GemTransactionSigner
import com.gemwallet.android.application.wallet_import.cases.ImportWalletService
import com.gemwallet.android.data.password.TinkPasswordStore
import com.gemwallet.android.data.password.TinkSecurityStore
import com.gemwallet.android.application.wallet.cases.SetCurrentWallet
import com.gemwallet.android.data.services.gemstone.wallets.PhraseAddressImportWalletService
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemAppStartService
import uniffi.gemstone.GemWalletService
import javax.inject.Singleton
import uniffi.gemstone.GemDeviceService
import uniffi.gemstone.GemTransferService

@InstallIn(SingletonComponent::class)
@Module
object InteractsModule {

    @Singleton
    @Provides
    fun provideValidatePhraseInteract(): ValidatePhraseOperator = GemValidatePhraseOperator()

    @Singleton
    @Provides
    fun provideCreateWalletInteract(): CreateWalletOperator = GemCreateWalletOperator()

    @Singleton
    @Provides
    fun provideMigrateKeystoreOperator(
        @ApplicationContext context: Context,
    ): MigrateKeystoreOperator = GemMigrateKeystoreOperator(context.dataDir.toString())

    @Singleton
    @Provides
    fun provideLoadPhraseInteract(
        @ApplicationContext context: Context
    ): LoadPrivateDataOperator =
        GemLoadPrivateDataOperator(context.dataDir.toString())

    @Singleton
    @Provides
    fun provideTransactionSigner(
        @ApplicationContext context: Context,
        passwordStore: PasswordStore,
        transferService: GemTransferService,
    ): GemTransactionSigner = KeystoreTransactionSigner(context.dataDir.toString(), passwordStore, transferService)

    @Singleton
    @Provides
    fun provideSignMessageOperator(
        @ApplicationContext context: Context,
    ): GemSignMessageOperator = GemSignMessageOperator(context.dataDir.toString())


    @Provides
    @Singleton
    fun providePasswordStore(@ApplicationContext context: Context): PasswordStore =
        TinkPasswordStore(context)

    @Provides
    @Singleton
    fun provideSecurityStore(@ApplicationContext context: Context): SecurityStore<Any> =
        TinkSecurityStore(context)

    @Singleton
    @Provides
    fun provideAddWalletInteract(
        walletService: GemWalletService,
        setCurrentWallet: SetCurrentWallet,
        appStartService: GemAppStartService,
        deviceService: GemDeviceService,
        walletImportSync: SyncWalletImport,
    ): ImportWalletService = PhraseAddressImportWalletService(
        walletService = walletService,
        setCurrentWallet = setCurrentWallet,
        appStartService = appStartService,
        deviceService = deviceService,
        walletImportSync = walletImportSync,
    )
}
