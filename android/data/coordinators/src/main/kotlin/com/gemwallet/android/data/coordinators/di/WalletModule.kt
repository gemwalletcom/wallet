package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.wallet.cases.DeleteWallet
import com.gemwallet.android.application.wallet.cases.GetWallet
import com.gemwallet.android.application.wallet.cases.GetWallets
import com.gemwallet.android.application.wallet.cases.GetAllWallets
import com.gemwallet.android.application.wallet.cases.GetWalletDetails
import com.gemwallet.android.application.wallet.cases.GetWalletSecretData
import com.gemwallet.android.application.wallet.cases.SetCurrentWallet
import com.gemwallet.android.application.wallet.cases.SetWalletName
import com.gemwallet.android.application.wallet.cases.SetWalletPinned
import com.gemwallet.android.blockchain.operators.DeleteKeyStoreOperator
import com.gemwallet.android.blockchain.operators.LoadPrivateDataOperator
import com.gemwallet.android.cases.addresses.RenameWalletAddresses
import com.gemwallet.android.data.coordinators.wallet.DeleteWalletImpl
import com.gemwallet.android.data.coordinators.wallet.GetWalletImpl
import com.gemwallet.android.data.coordinators.wallet.GetWalletsImpl
import com.gemwallet.android.data.coordinators.wallet.GetAllWalletsImpl
import com.gemwallet.android.data.coordinators.wallet.GetWalletDetailsImpl
import com.gemwallet.android.data.coordinators.wallet.GetWalletSecretDataImpl
import com.gemwallet.android.data.coordinators.wallet.SetCurrentWalletImpl
import com.gemwallet.android.data.coordinators.wallet.SetWalletNameImpl
import com.gemwallet.android.data.coordinators.wallet.SetWalletPinnedImpl
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.data.repositories.gemstone.GemstoneWalletStore
import com.gemwallet.android.data.repositories.session.SessionRepository
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemWalletService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object WalletModule {

    @Provides
    @Singleton
    fun provideGetWalletDetails(
        walletStore: GemstoneWalletStore,
    ): GetWalletDetails {
        return GetWalletDetailsImpl(walletStore)
    }

    @Provides
    @Singleton
    fun provideGetWallets(walletStore: GemstoneWalletStore): GetWallets = GetWalletsImpl(walletStore)

    @Provides
    @Singleton
    fun provideGetWallet(walletStore: GemstoneWalletStore): GetWallet = GetWalletImpl(walletStore)

    @Provides
    @Singleton
    fun provideGetAllWallets(
        sessionRepository: SessionRepository,
        walletStore: GemstoneWalletStore,
        walletService: GemWalletService,
    ): GetAllWallets {
        return GetAllWalletsImpl(sessionRepository, walletStore, walletService)
    }

    @Provides
    @Singleton
    fun provideSetWalletName(
        walletService: GemWalletService,
        renameWalletAddresses: RenameWalletAddresses,
    ): SetWalletName {
        return SetWalletNameImpl(walletService, renameWalletAddresses)
    }

    @Provides
    @Singleton
    fun provideGetWalletSecretData(
        walletStore: GemstoneWalletStore,
        passwordStore: PasswordStore,
        loadPrivateDataOperator: LoadPrivateDataOperator,
    ): GetWalletSecretData {
        return GetWalletSecretDataImpl(
            walletStore = walletStore,
            passwordStore = passwordStore,
            loadPrivateDataOperator = loadPrivateDataOperator,
        )
    }

    @Provides
    fun provideDeleteWallet(
        sessionRepository: SessionRepository,
        deleteKeyStoreOperator: DeleteKeyStoreOperator,
        walletService: GemWalletService,
        userConfig: UserConfig,
    ): DeleteWallet {
        return DeleteWalletImpl(sessionRepository, deleteKeyStoreOperator, walletService, userConfig)
    }

    @Provides
    fun provideSetWalletPinned(walletService: GemWalletService): SetWalletPinned {
        return SetWalletPinnedImpl(walletService)
    }

    @Provides
    fun provideSetCurrentWallet(
        sessionRepository: SessionRepository,
        walletStore: GemstoneWalletStore,
    ): SetCurrentWallet {
        return SetCurrentWalletImpl(sessionRepository, walletStore)
    }
}