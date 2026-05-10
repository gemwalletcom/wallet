package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.data.repositories.perpetual.PerpetualRepository
import com.gemwallet.android.data.repositories.perpetual.PerpetualRepositoryImpl
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.BalancesDao
import com.gemwallet.android.data.service.store.database.PerpetualDao
import com.gemwallet.android.data.service.store.database.PerpetualPositionDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object PerpetualModule {

    @Provides
    @Singleton
    fun providePerpetualRepository(
        perpetualDao: PerpetualDao,
        perpetualPositionDao: PerpetualPositionDao,
        assetsDao: AssetsDao,
        balancesDao: BalancesDao,
    ): PerpetualRepository {
        return PerpetualRepositoryImpl(
            perpetualDao = perpetualDao,
            perpetualPositionDao = perpetualPositionDao,
            assetsDao = assetsDao,
            balancesDao = balancesDao,
        )
    }
}
