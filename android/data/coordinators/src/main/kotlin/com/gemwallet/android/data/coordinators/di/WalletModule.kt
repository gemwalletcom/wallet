package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.wallet.coordinators.DeleteWallet
import com.gemwallet.android.application.wallet.coordinators.GetAllWallets
import com.gemwallet.android.application.wallet.coordinators.GetWalletDetails
import com.gemwallet.android.application.wallet.coordinators.GetWalletSecretData
import com.gemwallet.android.application.wallet.coordinators.SetCurrentWallet
import com.gemwallet.android.application.wallet.coordinators.SetWalletName
import com.gemwallet.android.application.wallet.coordinators.ToggleWalletPin
import com.gemwallet.android.application.wallet.coordinators.WalletIdGenerator
import com.gemwallet.android.blockchain.operators.DeleteKeyStoreOperator
import com.gemwallet.android.blockchain.operators.LoadPrivateDataOperator
import com.gemwallet.android.cases.addresses.RenameWalletAddresses
import com.gemwallet.android.cases.device.SyncSubscription
import com.gemwallet.android.data.coordinators.wallet.DeleteWalletImpl
import com.gemwallet.android.data.coordinators.wallet.GetAllWalletsImpl
import com.gemwallet.android.data.coordinators.wallet.GetWalletDetailsImpl
import com.gemwallet.android.data.coordinators.wallet.GetWalletSecretDataImpl
import com.gemwallet.android.data.coordinators.wallet.SetCurrentWalletImpl
import com.gemwallet.android.data.coordinators.wallet.SetWalletNameImpl
import com.gemwallet.android.data.coordinators.wallet.ToggleWalletPinImpl
import com.gemwallet.android.data.coordinators.wallet.WalletIdGeneratorImpl
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object WalletModule {
    @Provides
    @Singleton
    fun provideWalletIdGenerator(): WalletIdGenerator {
        return WalletIdGeneratorImpl()
    }

    @Provides
    @Singleton
    fun provideGetWalletDetails(
        walletsRepository: WalletsRepository,
    ): GetWalletDetails {
        return GetWalletDetailsImpl(walletsRepository)
    }

    @Provides
    @Singleton
    fun provideGetAllWallets(
        sessionRepository: SessionRepository,
        walletsRepository: WalletsRepository,
    ): GetAllWallets {
        return GetAllWalletsImpl(sessionRepository, walletsRepository)
    }

    @Provides
    @Singleton
    fun provideSetWalletName(
        walletsRepository: WalletsRepository,
        renameWalletAddresses: RenameWalletAddresses,
    ): SetWalletName {
        return SetWalletNameImpl(walletsRepository, renameWalletAddresses)
    }
    
    @Provides
    @Singleton
    fun provideGetWalletSecretData(
        walletsRepository: WalletsRepository,
        passwordStore: PasswordStore,
        loadPrivateDataOperator: LoadPrivateDataOperator,
    ): GetWalletSecretData {
        return GetWalletSecretDataImpl(
            walletsRepository = walletsRepository,
            passwordStore = passwordStore,
            loadPrivateDataOperator = loadPrivateDataOperator,
        )
    }

    @Provides
    fun provideDeleteWallet(
        sessionRepository: SessionRepository,
        walletsRepository: WalletsRepository,
        deleteKeyStoreOperator: DeleteKeyStoreOperator,
        syncSubscription: SyncSubscription,
    ): DeleteWallet {
        return DeleteWalletImpl(sessionRepository, walletsRepository, deleteKeyStoreOperator, syncSubscription)
    }

    @Provides
    fun provideToggleWalletPin(
        walletsRepository: WalletsRepository,
    ): ToggleWalletPin {
        return ToggleWalletPinImpl(walletsRepository)
    }

    @Provides
    fun provideSetCurrentWallet(
        sessionRepository: SessionRepository,
        walletsRepository: WalletsRepository,
    ): SetCurrentWallet {
        return SetCurrentWalletImpl(sessionRepository, walletsRepository)
    }
}