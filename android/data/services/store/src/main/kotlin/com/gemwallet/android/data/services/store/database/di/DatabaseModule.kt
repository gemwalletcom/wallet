package com.gemwallet.android.data.services.store.database.di

import android.content.Context
import androidx.room.Room
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.data.services.store.database.AccountsDao
import com.gemwallet.android.data.services.store.database.AddressesDao
import com.gemwallet.android.data.services.store.database.AssetListDao
import com.gemwallet.android.data.services.store.database.AssetsDao
import com.gemwallet.android.data.services.store.database.BalancesDao
import com.gemwallet.android.data.services.store.database.BannersDao
import com.gemwallet.android.data.services.store.database.ConnectionsDao
import com.gemwallet.android.data.services.store.database.ContactsDao
import com.gemwallet.android.data.services.store.database.FiatTransactionsDao
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.InAppNotificationsDao
import com.gemwallet.android.data.services.store.database.NftDao
import com.gemwallet.android.data.services.store.database.NodesDao
import com.gemwallet.android.data.services.store.database.PerpetualDao
import com.gemwallet.android.data.services.store.database.PerpetualPositionDao
import com.gemwallet.android.data.services.store.database.PriceAlertsDao
import com.gemwallet.android.data.services.store.database.PricesDao
import com.gemwallet.android.data.services.store.database.RoomStoreTransactionRunner
import com.gemwallet.android.data.services.store.database.SearchDao
import com.gemwallet.android.data.services.store.database.StakeDao
import com.gemwallet.android.data.services.store.database.StoreTransactionRunner
import com.gemwallet.android.data.services.store.database.SupportMessagesDao
import com.gemwallet.android.data.services.store.database.TransactionsDao
import com.gemwallet.android.data.services.store.database.WalletsDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object DatabaseModule {
    @Singleton
    @Provides
    fun provideRoom(@ApplicationContext context: Context, passwordStore: PasswordStore): GemDatabase = Room.databaseBuilder(
        context = context,
        klass = GemDatabase::class.java,
        name = "gem.db",
    )
        .addMigrations(Migration_41_42)
        .addMigrations(Migration_42_43)
        .addMigrations(Migration_43_44)
        .addMigrations(Migration_44_45)
        .addMigrations(Migration_45_46)
        .addMigrations(Migration_46_47)
        .addMigrations(Migration_47_48)
        .addMigrations(Migration_48_49)
        .addMigrations(Migration_49_50)
        .addMigrations(Migration_50_51)
        .addMigrations(Migration_51_52)
        .addMigrations(Migration_52_53)
        .addMigrations(Migration_53_54)
        .addMigrations(Migration_54_55)
        .addMigrations(Migration_55_56)
        .addMigrations(Migration_56_57)
        .addMigrations(Migration_57_58)
        .addMigrations(Migration_58_59)
        .addMigrations(Migration_59_60)
        .addMigrations(Migration_60_61)
        .addMigrations(Migration_61_62)
        .addMigrations(Migration_62_63)
        .addMigrations(Migration_63_64(context, passwordStore))
        .addMigrations(Migration_64_65)
        .addMigrations(Migration_65_66)
        .addMigrations(Migration_66_67)
        .addMigrations(Migration_67_68)
        .addMigrations(Migration_68_69)
        .addMigrations(Migration_69_70)
        .addMigrations(Migration_70_71)
        .addMigrations(*gemDatabaseMigrations(context))
        .build()

    @Singleton
    @Provides
    fun provideWalletsDao(db: GemDatabase): WalletsDao = db.walletsDao()

    @Singleton
    @Provides
    fun provideStoreTransactionRunner(runner: RoomStoreTransactionRunner): StoreTransactionRunner = runner

    @Singleton
    @Provides
    fun provideAccountsDao(db: GemDatabase): AccountsDao = db.accountsDao()

    @Singleton
    @Provides
    fun provideAddressesDao(db: GemDatabase): AddressesDao = db.addressesDao()

    @Singleton
    @Provides
    fun provideContactsDao(db: GemDatabase): ContactsDao = db.contactsDao()

    @Singleton
    @Provides
    fun provideAssetsDao(db: GemDatabase): AssetsDao = db.assetsDao()

    @Singleton
    @Provides
    fun provideBalancesDao(db: GemDatabase): BalancesDao = db.balancesDao()

    @Singleton
    @Provides
    fun providePricesDao(db: GemDatabase): PricesDao = db.pricesDao()

    @Singleton
    @Provides
    fun provideTransactionsDao(db: GemDatabase): TransactionsDao = db.transactionsDao()

    @Singleton
    @Provides
    fun provideConnectionsDao(db: GemDatabase): ConnectionsDao = db.connectionsDao()

    @Singleton
    @Provides
    fun provideStakeDao(db: GemDatabase): StakeDao = db.stakeDao()

    @Singleton
    @Provides
    fun provideNodeDao(db: GemDatabase): NodesDao = db.nodeDao()

    @Singleton
    @Provides
    fun provideBannersDao(db: GemDatabase): BannersDao = db.bannersDao()

    @Singleton
    @Provides
    fun providePriceAlertsDao(db: GemDatabase): PriceAlertsDao = db.priceAlertsDao()

    @Singleton
    @Provides
    fun provideNFTDao(db: GemDatabase): NftDao = db.nftDao()

    @Singleton
    @Provides
    fun provideSearchDao(db: GemDatabase): SearchDao = db.searchDao()

    @Singleton
    @Provides
    fun provideAssetListDao(db: GemDatabase): AssetListDao = db.assetListDao()

    @Singleton
    @Provides
    fun providePerpetualDao(db: GemDatabase): PerpetualDao = db.perpetualDao()

    @Singleton
    @Provides
    fun providePerpetualPositionDao(db: GemDatabase): PerpetualPositionDao = db.perpetualPositionDao()

    @Singleton
    @Provides
    fun provideFiatTransactionsDao(db: GemDatabase): FiatTransactionsDao = db.fiatTransactionsDao()

    @Singleton
    @Provides
    fun provideInAppNotificationsDao(db: GemDatabase): InAppNotificationsDao = db.inAppNotificationsDao()

    @Singleton
    @Provides
    fun provideSupportMessagesDao(db: GemDatabase): SupportMessagesDao = db.supportMessagesDao()
}
